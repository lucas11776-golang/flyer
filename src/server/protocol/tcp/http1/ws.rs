use std::net::SocketAddr;
use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use bytes::Bytes;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_tungstenite::{
    tungstenite::{protocol::Role::Server as RoleServer, Message, Utf8Bytes},
    WebSocketStream,
};

use crate::request::Request;
use crate::response::Response;
use crate::server::protocol::tcp::http1::Http1;
use crate::server::Server;
use crate::utils::mem::Instance;
use crate::websocket::{Event, SEC_WEB_SOCKET_ACCEPT_STATIC, Websocket, Writer, WriterInterface};

pub struct Ws {
    server: Instance<Server>,
    addr: SocketAddr,
}

#[derive(Clone)]
pub struct TcpWriter {
    sender: UnboundedSender<Message>,
}

impl TcpWriter {
    pub fn new(sender: UnboundedSender<Message>) -> Self {
        Self { sender }
    }
}

impl WriterInterface for TcpWriter {
    fn write(&mut self, data: Bytes) -> Result<()> {
        let text = Utf8Bytes::try_from(data)?;
        self.sender.send(Message::Text(text))?;
        Ok(())
    }

    fn write_binary(&mut self, data: Bytes) -> Result<()> {
        self.sender.send(Message::Binary(data))?;
        Ok(())
    }

    fn ping(&mut self, data: Bytes) -> Result<()> {
        self.sender.send(Message::Ping(data))?;
        Ok(())
    }

    fn pong(&mut self, data: Bytes) -> Result<()> {
        self.sender.send(Message::Pong(data))?;
        Ok(())
    }

    fn close(&mut self) -> Result<()> {
        self.sender.send(Message::Close(None))?;
        Ok(())
    }
}

impl Ws {
    pub fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self { server, addr }
    }

    pub async fn handle<RW>(&mut self, mut rw: BufReader<RW>, mut req: Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        Self::handshake(&mut rw, &mut req).await?;

        let ws_stream = WebSocketStream::from_raw_socket(rw, RoleServer, None).await;
        let (mut sink, stream) = ws_stream.split();

        let (tx, mut rx) = unbounded_channel::<Message>();

        let (req, route) = self.server.as_mut().on_websocket(req, Response::new()).await;
        let Some(route) = route else {
            return Ok(());
        };

        let websocket = (route.handler)(req, Websocket::new()).await;

        let writer_task = async move {
            while let Some(msg) = rx.recv().await {
                let is_close = matches!(msg, Message::Close(_));
                if sink.send(msg).await.is_err() || is_close {
                    break;
                }
            }
            let _ = sink.close().await;
        };

        let reader_task = Self::read_loop(stream, tx, websocket);

        tokio::join!(writer_task, reader_task);

        Ok(())
    }

    async fn read_loop<RW>(
        mut stream: SplitStream<WebSocketStream<BufReader<RW>>>,
        tx: UnboundedSender<Message>,
        ws: Websocket,
    ) where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        while let Some(Ok(msg)) = stream.next().await {
            let Some(callback) = &ws.event else {
                continue;
            };

            let writer = TcpWriter::new(tx.clone());

            let event = match msg {
                Message::Text(data) => Event::Text(data.into()),
                Message::Binary(bytes) => Event::Binary(bytes),
                Message::Ping(bytes) => Event::Ping(bytes),
                Message::Pong(bytes) => Event::Pong(bytes),
                Message::Close(frame) => {
                    todo!()
                },
                Message::Frame(_) => continue,
            };

            callback(event, Writer { instance: Box::new(writer) }).await;
        }
    }

    async fn handshake<RW>(rw: &mut BufReader<RW>, req: &mut Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    {
        let accept_key = Self::get_sec_web_socket_accept(&req.header("sec-websocket-key"));

        let res = Response::new()
            .status_code(101)
            .set_header("Upgrade", "websocket")
            .set_header("Connection", "Upgrade")
            .set_header("Sec-WebSocket-Accept", &accept_key);

        rw.write_all(Http1::serialize(&res).as_slice()).await?;
        rw.flush().await?;

        Ok(())
    }

    fn get_sec_web_socket_accept(key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(SEC_WEB_SOCKET_ACCEPT_STATIC.as_bytes());

        general_purpose::STANDARD.encode(hasher.finish())
    }
}