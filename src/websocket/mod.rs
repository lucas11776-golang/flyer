use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;

pub(crate) const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) type OnEvent = Box<dyn Fn(Event, Writer) -> BoxFuture<'static, ()> + Send + Sync>;

#[derive(Debug)]
pub struct Reason {
    pub code: u16,
    pub message: String,
}

pub enum Event {
    Ready(),
    Text(Bytes),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<Reason>),
}

pub trait WriterInterface: Send + Sync {
    fn write(&mut self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn write_binary(&mut self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn ping(&mut self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn pong(&mut self, data: Bytes) -> BoxFuture<'static, Result<()>>;
    fn close(&mut self) -> BoxFuture<'static, Result<()>>;
}

pub struct Writer {
    pub instance: Box<dyn WriterInterface>,
}


impl Writer {
    pub fn new(instance: impl WriterInterface + 'static) -> Self {
        return Self {
            instance: Box::new(instance)
        };
    }
}

impl Writer {
    async fn write(&mut self, data: Bytes) -> Result<()> {
        self
            .instance
            .write(data)
            .await
    }

    async fn write_binary(&mut self, data: Bytes) -> Result<()> {
        self
            .instance
            .write_binary(data)
            .await
    }

    async fn ping(&mut self, data: Bytes) -> Result<()> {
        self
            .instance
            .ping(data)
            .await
    }

    async fn pong(&mut self, data: Bytes) -> Result<()> {
        self
            .instance
            .pong(data)
            .await
    }

    async fn close(&mut self) -> Result<()> {
        self
            .instance
            .close()
            .await
    }
}


pub struct Websocket {
    pub(crate) event: Option<OnEvent>,
}

impl Websocket {
    pub fn new() -> Self {
        Self {
            event: None
        } 
    }

    pub fn on<C, Fut>(&mut self, callback: C)
    where
        C: Fn(Event, Writer) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.event = Some(Box::new(move |event, writer| Box::pin(callback(event, writer))));
    }
}