use std::{future::Future, io::Result, pin::Pin};

use futures_util::{
    future::BoxFuture, stream::SplitSink, FutureExt, SinkExt
};
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Message, Utf8Bytes};

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

pub(crate) type OnEvent = dyn Fn(Event) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub(crate) type Sending = Box<dyn FnMut(Message) -> Box<dyn Future<Output = ()> + Send + 'static> + Send + Sync + 'static>; 

pub struct Ws {
    pub(crate) event: Option<Box<OnEvent>>,
    pub(crate) writer: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl Ws {
    pub fn new(callback: tokio::sync::mpsc::UnboundedSender<Message>) -> Self {
        return Ws {
            event: None,
            writer: callback,
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
        self.writer.send(Message::Text(Utf8Bytes::from(String::from_utf8(data.to_vec()).unwrap()))).unwrap();
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}