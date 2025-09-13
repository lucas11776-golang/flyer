use std::future::Future;
use std::pin::Pin;

use futures_util::future::BoxFuture;
use futures_util::{FutureExt};
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;
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

pub struct Write<'a, R> {
    pub(crate) writer: SplitSink<WebSocketStream<Pin<&'a mut BufReader<R>>>, Message>,
}

impl <'a, R>Write<'a, R>
where
    R: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    pub async fn write(&mut self, data: Vec<u8>) {
    }

    pub async fn write_json<J>(&mut self, json: &J)
    where 
        J: ?Sized + Serialize
    {   
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) {
    }
}


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
        // TODO: add Fn(Write, Event)
        C: Fn(Event) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.event = Some(Box::new(move |event: Event| callback(event).boxed()));
    }
}