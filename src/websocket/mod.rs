use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;

use crate::utils::future::SendFuture;

pub(crate) const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) type OnEvent = Box<dyn Fn(Event, Socket) -> BoxFuture<'static, ()> + Send + Sync>;

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

#[allow(async_fn_in_trait)]
pub(crate) trait Writer: Send + Sync {
    async fn write(&self, data: Bytes) -> Result<()>;
    async fn write_binary(&self, data: Bytes) -> Result<()>;
    async fn ping(&self, data: Bytes) -> Result<()>;
    async fn pong(&self, data: Bytes) ->  Result<()>;
    async fn close(&self) -> Result<()>;
}


pub trait WsInterface: Send + Sync {
    fn write(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn write_binary(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn ping(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn pong(&self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn close(&self) -> BoxFuture<'static, Result<()>>;
}

pub struct Socket {
    inner: Arc<dyn WsInterface>
}

impl Socket {
    pub fn new(writer: impl WsInterface + 'static) -> Self {
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

// pub(crate) struct WriterWrapper<T: Writer + 'static> {
//     pub instance: Arc<T>,
// }

// impl <T: Writer + 'static>WriterWrapper<T> {
//     pub fn new(instance: T) -> Self {
//         Self {
//             instance: Arc::new(instance),
//         }
//     }
// }

// impl<T: Writer + 'static> WsInterface for WriterWrapper<T> {
//     fn write(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
//         let instance = Arc::clone(&self.instance);

//         return Box::pin(async move {
//             SendFuture(instance.write(data)).await
//         });
//     }
    
//     fn write_binary(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
//         todo!()
//     }
    
//     fn ping(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
//         todo!()
//     }
    
//     fn pong(&self, data: Bytes) -> BoxFuture<'static, Result<()>> {
//         todo!()
//     }
    
//     fn close(&self) -> BoxFuture<'static, Result<()>> {
//         todo!()
//     }
// }


pub struct Websocket {
    pub(crate) event: Option<OnEvent>,
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