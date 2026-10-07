use std::{any::Any, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;

use crate::utils::mem::Instance;

pub(crate) const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

impl WebsocketEventCallback {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct Websocket {
    pub(crate) events: Instance<WebsocketEventCallback>,
    guards: Vec<Box<dyn Any + Send + Sync>>,
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

#[derive(Default)]
pub(crate) struct WebsocketEventCallback {
    pub ready: Option<Box<dyn Fn(Socket) -> BoxFuture<'static, ()> + Send + Sync>>,
    pub text: Option<Box<dyn Fn(Bytes, Socket) -> BoxFuture<'static, ()> + Send + Sync>>,
    pub binary: Option<Box<dyn Fn(Bytes, Socket) -> BoxFuture<'static, ()> + Send + Sync>>,
    pub ping: Option<Box<dyn Fn(Bytes, Socket) -> BoxFuture<'static, ()> + Send + Sync>>,
    pub pong: Option<Box<dyn Fn(Bytes, Socket) -> BoxFuture<'static, ()> + Send + Sync>>,
    pub close: Option<Box<dyn Fn(Option<Reason>) -> BoxFuture<'static, ()> + Send + Sync>>,
    
}

impl Websocket {
    // pub fn new(event: Arc<WebsocketEventCallback>, writer: Arc<impl Writer + 'static>) -> Self {
    pub(crate) fn new(event: Instance<WebsocketEventCallback>) -> Self {
        Self {
            events: event,
            guards: Default::default(),
        } 
    }

    pub fn writer(&self) -> Arc<dyn Writer + 'static> {
        todo!()
    }

    pub fn keep_alive<T: Send + Sync + 'static>(mut self, resource: T) -> Self {
        self
            .guards
            .push(Box::new(resource));
        self
    }
    
    pub fn ready<C, Fut>(self, callback: C) -> Self
    where
        C: Fn(Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.events.as_mut().ready = Some(Box::new(move |socket| {
            Box::pin(callback(socket))
        }));
        self
    }

    pub fn text<C, Fut>(self, callback: C) -> Self 
    where
        C: Fn(Bytes, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.events.as_mut().text = Some(Box::new(move |event, socket| {
            Box::pin(callback(event, socket))
        }));
        self
    }

    pub fn binary<C, Fut>(self, callback: C) -> Self 
    where
        C: Fn(Bytes, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.events.as_mut().binary = Some(Box::new(move |event, socket| {
            Box::pin(callback(event, socket))
        }));
        self
    }

    pub fn ping<C, Fut>(self, callback: C) -> Self 
    where
        C: Fn(Bytes, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.events.as_mut().ping = Some(Box::new(move |event, socket| {
            Box::pin(callback(event, socket))
        }));
        self
    }

    pub fn pong<C, Fut>(self, callback: C) -> Self 
    where
        C: Fn(Bytes, Socket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.events.as_mut().pong = Some(Box::new(move |event, socket| {
            Box::pin(callback(event, socket))
        }));
        self
    }

    pub fn close<C, Fut>(self, callback: C) -> Self 
    where
        C: Fn(Option<Reason>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.events.as_mut().close = Some(Box::new(move |reason| {
            Box::pin(callback(reason))
        }));
        self
    }
}