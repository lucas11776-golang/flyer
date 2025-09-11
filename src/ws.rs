use std::future::Future;
use std::io::Result;

use futures_util::future::BoxFuture;
use futures_util::{FutureExt};
use serde::Serialize;
use tungstenite::Message;

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

#[derive(Debug)]
pub struct Reason {
    pub code: u16,
    pub message: String,
}

pub type OnReady = dyn Fn(Ws) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub type OnMessage = dyn Fn(Ws, Vec<u8>) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub type OnPing = dyn Fn(Ws, Vec<u8>) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub type OnPong = dyn Fn(Ws, Vec<u8>) -> BoxFuture<'static, ()> + Send + Sync + 'static;
pub type OnClose = dyn Fn(Option<Reason>) -> BoxFuture<'static, ()> + Send + Sync + 'static;

pub trait Writer {
    fn write(&mut self, item: Message) -> impl std::future::Future<Output = ()> + Send;
}

pub struct WsWrite {

}

impl Writer for WsWrite {
    async fn write(&mut self, item: Message) {
        todo!()
    }
}

#[derive(Default)]
pub struct Ws {
    pub ready: Option<Box<OnReady>>,
    pub message: Option<Box<OnMessage>>,
    pub ping: Option<Box<OnPing>>,
    pub pong: Option<Box<OnPong>>,
    pub close: Option<Box<OnClose>>,
}

// TODO: find way to nest on_message to ready...
impl <'a>Ws {
    pub fn new() -> Self {
        return Self {
            ready: None,
            message: None,
            ping: None,
            pong: None,
            close: None,
        }
    }

    pub fn on_ready<F, C>(&mut self, callback: C)
    where
        C: Fn(Ws) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.ready = Some(Box::new(move |ws: Ws| callback(ws).boxed()));
    }

    pub fn on_message<F, C>(&mut self, callback: C)
    where
        C: Fn(Ws, Vec<u8>) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.message = Some(Box::new(move |ws: Ws, data: Vec<u8>| callback(ws, data).boxed()));
    }

    pub fn on_ping<F, C>(&mut self, callback: C)
    where
        C: Fn(Ws, Vec<u8>) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.ping = Some(Box::new(move |ws: Ws, data: Vec<u8>| callback(ws, data).boxed()));
    }

    pub fn on_pong<F, C>(&mut self, callback: C)
    where
        C: Fn(Ws, Vec<u8>) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.pong = Some(Box::new(move |ws: Ws, data: Vec<u8>| callback(ws, data).boxed()));
    }

    pub fn on_close<F, C>(&mut self, callback: C)
    where
        C: Fn(Option<Reason>) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        self.close = Some(Box::new(move |reason: Option<Reason>| callback(reason).boxed()));
    }

    pub async fn write(&mut self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    pub async fn write_json<J>(&mut self, json: &J) -> Result<()>
    where 
        J: ?Sized + Serialize
    {
        Ok(())
    }

    pub async fn write_binary(&mut self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }
}