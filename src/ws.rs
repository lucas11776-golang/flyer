use std::future::Future;
use std::pin::Pin;

use futures_util::future::BoxFuture;
use futures_util::{FutureExt};
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Message, Utf8Bytes};
use futures_util::stream::SplitSink;

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

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

pub type OnEvent = dyn Fn(Event) -> BoxFuture<'static, ()> + Send + Sync + 'static;


// pub type WsSend = dyn FnMut(Message) -> dyn Future<Output = ()>;
// pub type WsSend = dyn FnMut(Message) -> dyn Future<Output = ()>;
// pub type WsSend = dyn Fn(Message) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub type WsSend = dyn Fn(Message) -> () + Send + Sync;

pub struct Ws {
    pub(crate) event: Option<Box<OnEvent>>,
    pub(crate) send: Option<Box<WsSend>>
}

impl Ws {
    pub fn new() -> Self {
        return Ws {
            event: None,
            send: None,
        } 
    }

    pub fn on<'a, F, C>(&mut self, callback: C)
    where
        C: Fn(Event) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.event = Some(Box::new(move |event: Event| callback(event).boxed()));
    }

    pub async fn write(&mut self, data: Vec<u8>) {
        self.send.as_ref().unwrap()(Message::Text(Utf8Bytes::from(String::from_utf8(data.to_vec()).unwrap())));
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}