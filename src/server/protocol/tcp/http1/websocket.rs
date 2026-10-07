use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use bytes::Bytes;
use futures::{
    prelude::{future::BoxFuture, stream::SplitSink},
    SinkExt, StreamExt,
};
use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio_tungstenite::{
    tungstenite::{protocol::Role::Server as RoleServer, Message, Utf8Bytes},
    WebSocketStream,
};
use tungstenite::protocol::frame;

use crate::{
    Websocket, request::Request, response::Response, server::{Server, protocol::tcp::http1::Http1}, utils::mem::Instance, websocket::{self, Event, Reason, SEC_WEB_SOCKET_ACCEPT_STATIC, Socket, WebsocketEventCallback, Writer},
};

pub struct Http1Websocket {
    server: Instance<Server>,
    _addr: SocketAddr,
}

#[derive(Clone)]
pub struct Http1WebsocketWriter<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    inner: Instance<SplitSink<WebSocketStream<BufReader<RW>>, Message>>,
}

impl<RW> Http1WebsocketWriter<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    #[inline]
    pub fn new(instance: Instance<SplitSink<WebSocketStream<BufReader<RW>>, Message>>) -> Self {
        Self {
            inner: instance,
        }
    }

    fn send(&self, msg: Message) -> BoxFuture<'static, Result<()>> {
        let inner = self.inner.clone();
        Box::pin(async move {
            inner
                .as_mut()
                .send(msg)
                .await
                .map_err(Into::into)
        })
    }
}

impl<RW> Writer for Http1WebsocketWriter<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
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
            server: server,
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

        let writer = Http1WebsocketWriter::new(Instance::from_mut(&mut sink));
        let socket = websocket::Socket::new(writer);


        let mut events = WebsocketEventCallback::new();

        let websocket = Websocket::new(Instance((&mut events).into()));

        (route.handler)(req, websocket).await;

        if let Some(ref cb) = events.ready {
            tokio::spawn(cb(socket.clone()));
        }

        while let Some(Ok(msg)) = stream.next().await {
            match msg {
                Message::Text(data) => {
                    if let Some(ref cb) = events.text {
                        cb(data.into(), socket.clone()).await;
                    }
                },
                Message::Binary(data) => {
                    if let Some(ref cb) = events.binary {
                        cb(data.into(), socket.clone()).await;
                    }
                },
                Message::Ping(data) => {
                    if let Some(ref cb) = events.ping {
                        cb(data.into(), socket.clone()).await;
                    }
                },
                Message::Pong(data) => {
                    if let Some(ref cb) = events.pong {
                        cb(data.into(), socket.clone()).await;
                    }
                },
                Message::Close(frame) => {
                    if let Some(ref cb) = events.close {
                        cb(frame.map(|f| Reason::new(f.code.into(), f.reason.into()))).await;
                    }
                },
                Message::Frame(_frame) => {},
            }
        }

        Ok(())
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

        Http1::write_response(rw, &mut res).await?;

        Ok(res)
    }

    fn get_sec_web_socket_accept(key: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(SEC_WEB_SOCKET_ACCEPT_STATIC.as_bytes());
        let hash = hasher.finish();

        let mut buf = [0u8; 28];
        let len = general_purpose::STANDARD
            .encode_slice(hash, &mut buf)
            .expect("28-byte buffer fits 20-byte SHA-1 base64 output");

        std::str::from_utf8(&buf[..len])
            .unwrap()
            .to_string()
    }
}