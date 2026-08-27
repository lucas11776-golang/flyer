use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;

pub(crate) const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) type OnEvent = Box<dyn Fn(Event, Socket) -> BoxFuture<'static, ()> + Send + Sync>;

pub struct Websocket {
    pub(crate) event: Option<OnEvent>,
}

#[derive(Debug)]
pub struct Reason {
    pub code: u16,
    pub message: Bytes,
}

impl Reason {
    pub fn new(code: u16, message: Bytes) -> Self {
        Self {
            code: code,
            message: message,
        }
    }
}

pub enum Event {
    Ready(),
    Text(Bytes),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<Reason>),
}

pub trait Writer: Send + Sync {
    fn write(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn write_binary(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn ping(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn pong(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    // TODO: need to implement message when closing.
    fn close(&self) -> BoxFuture<'static, Result<()>>;
}

#[derive(Clone)]
pub struct Socket {
    inner: Arc<dyn Writer>
}

impl Socket {
    pub fn new(writer: impl Writer + 'static) -> Self {
        Self {
            inner: Arc::new(writer),
        }
    }
}

impl Socket {
    pub async fn write(&self, data: Bytes) -> Result<()> {
        self
            .inner
            .write(data)
            .await
    }

    pub async fn write_binary(&self, data: Bytes) -> Result<()> {
        self
            .inner
            .write_binary(data)
            .await
    }

    pub async fn ping(&self, data: Bytes) -> Result<()> {
        self
            .inner
            .ping(data)
            .await
    }

    pub async fn pong(&self, data: Bytes) ->  Result<()> {
        self
            .inner
            .pong(data)
            .await
    }

    pub async fn close(&self) -> Result<()> {
        self
            .inner
            .close()
            .await
    }
}

impl Websocket {
    pub fn new() -> Self {
        Self {
            event: None
        } 
    }

    pub fn on<C, Fut>(mut self, callback: C) -> Self
    where
        C: Fn(Event, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.event = Some(Box::new(move |event, writer| Box::pin(callback(event, writer))));

        return self;
    }
}