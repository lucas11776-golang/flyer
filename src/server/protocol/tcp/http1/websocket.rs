use std::net::SocketAddr;

use anyhow::Result;
use base64::{Engine, engine::general_purpose};
use bytes::Bytes;
use futures::{
    SinkExt, StreamExt,
    prelude::future::BoxFuture,
};
use openssl::sha::Sha1;
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader},
    sync::mpsc,
};
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{Message, Utf8Bytes, protocol::Role::Server as RoleServer},
};

use crate::{
    Websocket,
    request::Request,
    response::Response,
    server::{Server, protocol::tcp::http1::Http1},
    utils::mem::Instance,
    websocket::{self, Reason, SEC_WEB_SOCKET_ACCEPT_STATIC, WebsocketEventCallback, Writer},
};

pub struct Http1Websocket {
    server: Instance<Server>,
    _addr: SocketAddr,
}

/// High-performance, thread-safe WebSocket writer backed by an asynchronous channel.
#[derive(Clone)]
pub struct Http1WebsocketWriter {
    tx: mpsc::Sender<Message>,
}

impl Http1WebsocketWriter {
    #[inline]
    pub fn new(tx: mpsc::Sender<Message>) -> Self {
        Self { tx }
    }

    #[inline]
    fn send(&self, msg: Message) -> BoxFuture<'static, Result<()>> {
        let tx = self.tx.clone();
        Box::pin(async move {
            tx.send(msg)
                .await
                .map_err(|_| anyhow::anyhow!("WebSocket connection closed"))
        })
    }
}

impl Writer for Http1WebsocketWriter {
    fn write(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
        match Utf8Bytes::try_from(data) {
            Ok(utf8) => self.send(Message::Text(utf8)),
            Err(err) => Box::pin(async move { Err(err.into()) }),
        }
    }

    fn write_binary(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
        self.send(Message::Binary(data))
    }

    fn ping(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
        self.send(Message::Ping(data))
    }

    fn pong(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
        self.send(Message::Pong(data))
    }

    fn close(&self) -> BoxFuture<'static, Result<()>> {
        self.send(Message::Close(None))
    }
}

impl Http1Websocket {
    #[inline]
    pub fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self {
            server,
            _addr: addr,
        }
    }

    pub async fn handle<RW>(&mut self, mut rw: BufReader<RW>, mut req: Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let res = Self::handshake(&mut rw, &mut req).await?;

        let Some((req, res, route)) = self.server.as_mut().on_websocket(req, res).await else {
            return Ok(());
        };

        let (mut sink, mut stream) = WebSocketStream::from_raw_socket(rw, RoleServer, None)
            .await
            .split();

        // Channel queue for non-blocking concurrent writes with backpressure buffer
        let (tx, mut rx) = mpsc::channel::<Message>(128);

        // Background write pump eliminates lock contention and unsafe pointer aliasing
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        });

        let writer = Http1WebsocketWriter::new(tx);
        let socket = websocket::Socket::new(writer);

        let mut events = WebsocketEventCallback::new();

        let _websocket = (route.handler)(
            req,
            Websocket::new(socket.clone(), Instance((&mut events).into())),
        )
        .await;

        if let Some(ref cb) = events.ready {
            tokio::spawn(cb(socket.clone()));
        }

        while let Some(msg_res) = stream.next().await {
            let msg = match msg_res {
                Ok(m) => m,
                Err(_) => break,
            };

            match msg {
                Message::Text(data) => {
                    if let Some(ref cb) = events.text {
                        cb(data.into(), socket.clone()).await;
                    }
                }
                Message::Binary(data) => {
                    if let Some(ref cb) = events.binary {
                        cb(data.into(), socket.clone()).await;
                    }
                }
                Message::Ping(data) => {
                    if let Some(ref cb) = events.ping {
                        cb(data.into(), socket.clone()).await;
                    }
                }
                Message::Pong(data) => {
                    if let Some(ref cb) = events.pong {
                        cb(data.into(), socket.clone()).await;
                    }
                }
                Message::Close(frame) => {
                    if let Some(ref cb) = events.close {
                        cb(frame.map(|f| Reason::new(f.code.into(), f.reason.into()))).await;
                    }
                    break;
                }
                Message::Frame(_) => {}
            }
        }

        Ok(())
    }

    async fn handshake<RW>(rw: &mut BufReader<RW>, req: &mut Request) -> Result<Response>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let accept_buf = Self::get_sec_web_socket_accept(&req.header("sec-websocket-key"));
        let accept_key = std::str::from_utf8(&accept_buf)?;

        let mut res = Response::new()
            .status_code(101)
            .set_header("Upgrade", "websocket")
            .set_header("Connection", "Upgrade")
            .set_header("Sec-WebSocket-Accept", accept_key);

        Http1::write_response(rw, &mut res).await?;

        Ok(res)
    }

    /// Computes Sec-WebSocket-Accept with zero heap allocations.
    #[inline]
    fn get_sec_web_socket_accept(key: &str) -> [u8; 28] {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(SEC_WEB_SOCKET_ACCEPT_STATIC.as_bytes());
        let hash = hasher.finish();

        let mut buf = [0u8; 28];
        general_purpose::STANDARD
            .encode_slice(hash, &mut buf)
            .expect("28-byte buffer fits 20-byte SHA-1 base64 output");

        buf
    }
}