use std::future::Future;
use std::pin::Pin;

use futures_util::{
    FutureExt,
    future::BoxFuture
};
use serde::Serialize;
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use tungstenite::{Message, Utf8Bytes};

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
pub struct Reason {
    pub code: u16,
    pub message: String,
}

pub trait WsSend: Send {
    fn send(&self, message: Message) -> ();
}

pub enum Event {
    Ready(),
    Message(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<Reason>),
}

pub(crate) type OnEvent = dyn Fn(Event) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub(crate) type TSend = Box<dyn FnMut(Message) -> Pin<Box<dyn Future<Output = std::result::Result<(), tungstenite::Error>> + Send>> + Send>;

pub struct Ws {
    pub(crate) event: Option<Box<OnEvent>>,
    pub(crate) writer: Box<TSend>
}

impl <'a>Ws {
    pub async fn new<R>(writer: Box<TSend>) -> Self
    where
        R: AsyncRead + AsyncWrite+ Unpin + Send + Sync
    {
        return Ws {
            event: None,
            writer: writer
        } 
    }

    pub fn on<F, C>(&mut self, callback: C)
    where
        C: Fn(Event) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.event = Some(Box::new(move |event: Event| callback(event).boxed()));
    }

    pub async fn write(&mut self, data: Vec<u8>) {
        self.writer.as_mut()(Message::Text(Utf8Bytes::from(String::from_utf8(data.to_vec()).unwrap()))).await.unwrap();
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}