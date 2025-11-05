use serde::Serialize;
use tungstenite::{Message, Utf8Bytes};
use futures_util::future::BoxFuture;
use futures::future::{Future, FutureExt};

pub(crate) type OnEvent = dyn Fn(Event, Writer) -> BoxFuture<'static, ()> + Send + Sync + 'static;

#[derive(Debug)]
pub struct Reason {
    pub code: u16,
    pub message: String,
}

pub enum Event {
    Ready(),
    Message(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<Reason>),
}

pub struct Ws {
    pub(crate) event: Option<Box<OnEvent>>,
}

pub struct Writer {
    pub(crate) sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl Ws {
    pub fn new() -> Self {
        return Ws {
            event: None
        } 
    }

    pub fn on<F, C>(&mut self, callback: C)
    where
        C: Fn(Event, Writer) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.event = Some(Box::new(move |event: Event, writer: Writer| callback(event, writer).boxed()));
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}

impl Writer {
    pub(crate) fn new(sender: tokio::sync::mpsc::UnboundedSender<Message>) -> Self {
        return Self {
            sender: sender
        }
    }

    pub async fn write(&mut self, data: Vec<u8>) {
        self.sender.send(Message::Text(Utf8Bytes::from(String::from_utf8(data.to_vec()).unwrap()))).unwrap();
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}