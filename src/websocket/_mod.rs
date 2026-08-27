use anyhow::Result;
use bytes::Bytes;
use futures::future::BoxFuture;

pub(crate) const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) type OnEvent = Box<dyn Fn(Event, Writer) -> BoxFuture<'static, ()> + Send + Sync>;

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

pub trait WriterInterface: Send + Sync {
    fn write(&self, data: Bytes) -> Result<()>;
    fn write_binary(&self, data: Bytes) -> Result<()>;
    fn ping(&self, data: Bytes) -> Result<()>;
    fn pong(&self, data: Bytes) ->  Result<()>;
    fn close(&self) -> Result<()>;
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

impl WriterInterface for Writer {
    fn write(&self, data: Bytes) -> Result<()> {
        self.instance.write(data)
    }

    fn write_binary(&self, data: Bytes) -> Result<()> {
        self.instance.write_binary(data)
    }

    fn ping(&self, data: Bytes) -> Result<()> {
        self.instance.ping(data)
    }

    fn pong(&self, data: Bytes) ->  Result<()> {
        self.instance.pong(data)
    }

    fn close(&self) -> Result<()> {
        self.instance.close()
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

    pub fn on<C, Fut>(mut self, callback: C) -> Self
    where
        C: Fn(Event, Writer) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.event = Some(Box::new(move |event, writer| Box::pin(callback(event, writer))));

        return self;
    }
}