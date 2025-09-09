
use std::cell::UnsafeCell;
use std::future::Future;
use std::{fmt::Debug, io::{Error, Result}, pin::Pin};

use bytes::Bytes;
use futures_util::{stream::SplitSink, SinkExt};
use serde::Serialize;
use tokio::{io::{AsyncRead, AsyncWrite, BufReader}};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Message, Utf8Bytes};

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub type OnReady = fn (ws: &mut Ws);
pub type OnMessage = fn (data: Vec<u8>);
pub type OnPing = fn (data: Vec<u8>);
pub type OnPong = fn (data: Vec<u8>);
pub type OnClose = fn (code: u16);
pub type OnError = fn (error: Error);

pub trait Rw: Debug + Send + Sync
{
    fn send(&mut self, item: Message) -> impl Future<Output = Result<()>>
    where 
        Self: Sized;
}


#[derive(Debug)]
pub(crate) struct Writer<'a, R> {
    // pub(crate) writer: &'a mut Pin<Box<SplitSink<WebSocketStream<Pin<&'a mut BufReader<R>>>, Message>>>
    pub(crate) writer: SplitSink<WebSocketStream<Pin<&'a mut BufReader<R>>>, Message>,
}


// TODO: life time final boss...
impl <'a, R>Rw for Writer<'a, R>
where
    R: AsyncRead + AsyncWrite + Unpin + Send + Debug
{
    async fn send(&mut self, item: Message) -> Result<()>
    // where
    //     Self: Sized
    {
        return Ok(self.writer.send(item).await.unwrap());
    }
}


#[derive(Debug)]
pub struct Ws
{
    // pub(crate) rw: Box<&'a dyn Rw>,
    pub(crate) rw: Pin<Box<dyn Rw>>,
    pub(crate) ready: Option<OnReady>,
    pub(crate) message: Option<OnMessage>,
    pub(crate) ping: Option<OnPing>,
    pub(crate) pong: Option<OnPong>,
    pub(crate) close: Option<OnClose>,
    pub(crate) error: Option<OnError>,
}

impl <'a>Ws {
    pub fn on_ready(&mut self, callback: OnReady) {
        self.ready = Some(callback);
    }

    pub fn on_message(&mut self, callback: OnMessage) {
        self.message = Some(callback);
    }

    pub fn on_ping(&mut self, callback: OnPing) {
        self.ping = Some(callback);
    }

    pub fn on_pong(&mut self, callback: OnPong) {
        self.pong = Some(callback);
    }


    pub fn on_close(&mut self, callback: OnClose) {
        self.close = Some(callback);
    }

    pub fn on_error(&mut self, callback: OnError) {
        self.error = Some(callback);
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        
        


        // self.rw.as_mut().send(Message::Text(Utf8Bytes::from(data.into())));
        



        
        // self.send(Box::pin(self.rw), item);

        // self.rw.as_mut().send(Message::Text(Utf8Bytes::from(data.into())));

        // self.rw.as_mut().send(Message::Text(Utf8Bytes::from(data.into())));

        // self.rw.as_ref().send();

        Ok(())
    }

    async fn send<T: Rw>(&mut self, rw: Pin<Box<T>>, item: Message) -> Result<()> {



        // rw.as_ref().send(item);

        // rw.send(item).await.unwrap();

        Ok(())
    }

    pub fn write_string(&mut self, data: String) -> Result<()> {
        Ok(())
    }


    pub fn write_json<J>(&mut self, object: &J) -> Result<()>
    where 
        J: ?Sized + Serialize
    {
        Ok(())
    }

    pub fn write_baniry(&mut self, data: Bytes) -> Result<()> {
        Ok(())
    }
}