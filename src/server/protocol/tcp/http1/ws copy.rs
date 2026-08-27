use std::net::SocketAddr;

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use bytes::Bytes;
use futures::{prelude::stream::SplitSink, stream::SplitStream, SinkExt, StreamExt};
use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
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
use crate::websocket::{Event, Reason, SEC_WEB_SOCKET_ACCEPT_STATIC, Websocket, Writer};

pub struct Http1Websocket {
    server: Instance<Server>,
    _addr: SocketAddr,
}

#[derive(Clone)]
pub struct TcpWriter<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    inner: Instance<SplitSink<WebSocketStream<BufReader<RW>>, Message>>,
}

impl <RW>TcpWriter<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub fn new(instance: Instance<SplitSink<WebSocketStream<BufReader<RW>>, Message>>) -> Self {
        Self { inner: instance }
    }
}

impl <RW>crate::websocket::WsInterface for TcpWriter<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    fn write(&self, data: Bytes) -> futures::prelude::future::BoxFuture<'static, Result<()>> {
        todo!()
    }
    
    fn write_binary(&self, data: Bytes) -> futures::prelude::future::BoxFuture<'static, Result<()>> {
        todo!()
    }
    
    fn ping(&self, data: Bytes) -> futures::prelude::future::BoxFuture<'static, Result<()>> {
        todo!()
    }
    
    fn pong(&self, data: Bytes) -> futures::prelude::future::BoxFuture<'static, Result<()>> {
        todo!()
    }
    
    fn close(&self) -> futures::prelude::future::BoxFuture<'static, Result<()>> {
        todo!()
    }
    // fn write(&self, data: Bytes) -> Result<()> {
    //     let text = Utf8Bytes::try_from(data)?;
    //     self.sender.send(Message::Text(text))?;
    //     Ok(())
    // }

    // fn write_binary(&self, data: Bytes) -> Result<()> {
    //     self.sender.send(Message::Binary(data))?;
    //     Ok(())
    // }

    // fn ping(&self, data: Bytes) -> Result<()> {
    //     self.sender.send(Message::Ping(data))?;
    //     Ok(())
    // }

    // fn pong(&self, data: Bytes) -> Result<()> {
    //     self.sender.send(Message::Pong(data))?;
    //     Ok(())
    // }

    // fn close(&self) -> Result<()> {
    //     self.sender.send(Message::Close(None))?;
    //     Ok(())
    // }
}

impl Http1Websocket {
    pub fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self {
            server: server,
            _addr: addr
        }
    }

    // TODO: need to refactor but moving away from `unbounded_channel`
    pub async fn handle<RW>(&mut self, mut rw: BufReader<RW>, mut req: Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let res = Self::handshake(&mut rw, &mut req)
            .await
            .unwrap();
        
        let result = self
            .server
            .as_mut()
            .on_websocket(req, res)
            .await;

        let Some(websocket) = result else {
            return Ok(());
        };

        let ws_stream = WebSocketStream::from_raw_socket(rw, RoleServer, None).await;
        let (mut sink, stream) = ws_stream.split();

        // let sink: futures::prelude::stream::SplitSink<WebSocketStream<BufReader<RW>>, Message> = sink;

        let (tx, mut rx) = unbounded_channel::<Message>();

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

        println!("WEBSOCKET CONNECTION KILLED");

        Ok(())
    }

    async fn read_loop<RW>(
        mut stream: SplitStream<WebSocketStream<BufReader<RW>>>,
        tx: UnboundedSender<Message>,
        ws: Websocket,
    ) where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {




        todo!()



        // let writer = TcpWriter::new(tx);

        // while let Some(Ok(msg)) = stream.next().await {
        //     let Some(callback) = &ws.event else {
        //         continue;
        //     };

        //     let event = match msg {
        //         Message::Text(data) => Event::Text(data.into()),
        //         Message::Binary(bytes) => Event::Binary(bytes),
        //         Message::Ping(bytes) => Event::Ping(bytes),
        //         Message::Pong(bytes) => Event::Pong(bytes),
        //         Message::Close(frame) => Event::Close(frame.map(|f| Reason::new(f.code.into(), f.reason.into()))),
        //         Message::Frame(_) => continue,
        //     };


        //     todo!()
        //     // callback(event, Writer::new(writer.clone())).await;
        // }
    }

    async fn handshake<RW>(rw: &mut BufReader<RW>, req: &mut Request) -> Result<Response>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let accept_key = Self::get_sec_web_socket_accept(&req.header("sec-websocket-key"));

        let mut res = Response::new()
            .status_code(101)
            .set_header("Upgrade", "websocket")
            .set_header("Connection", "Upgrade")
            .set_header("Sec-WebSocket-Accept", &accept_key);

        Http1::write_response(rw, &mut res)
            .await
            .unwrap();

        Ok(res)
    }

    fn get_sec_web_socket_accept(key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(SEC_WEB_SOCKET_ACCEPT_STATIC.as_bytes());

        general_purpose::STANDARD.encode(hasher.finish())
    }
}