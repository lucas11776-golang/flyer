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

pub struct Ws {
    pub(crate) event: Option<Box<OnEvent>>,
}

impl <'a>Ws {
    pub fn new() -> Self {
        return Ws {
            event: None,
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
        // self.writer.as_mut()(Message::Text(Utf8Bytes::from(String::from_utf8(data.to_vec()).unwrap()))).await.unwrap();
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}