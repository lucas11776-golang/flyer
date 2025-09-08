use std::{fmt::Debug, io::{Error, Result}, pin::Pin};

use futures_util::stream::SplitSink;
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;


pub const SecWebSocketAcceptStatic: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

// TODO: fix
pub trait RW: Send + Debug {
    // fn read(&mut self) -> Result<Vec<u8>>;
    fn write(&mut self, payload: Vec<u8>) -> Result<()>;
}

pub type OnReady = fn (ws: &mut Ws);
pub type OnMessage = fn (data: Vec<u8>);
pub type OnPing = fn (data: Vec<u8>);
pub type OnPong = fn (data: Vec<u8>);
pub type OnClose = fn (code: u16);
pub type OnError = fn (error: Error);


#[derive(Debug)]
pub struct Ws {
    pub(crate) rw: Pin<Box<dyn RW>>,
    pub(crate) ready: Option<OnReady>,
    pub(crate) message: Option<OnMessage>,
    pub(crate) ping: Option<OnPing>,
    pub(crate) pong: Option<OnPong>,
    pub(crate) close: Option<OnClose>,
    pub(crate) error: Option<OnError>,
}

impl Ws {
    pub async fn new(rw: Pin<Box<dyn RW>>) -> Self {
        return Ws {
            rw: rw,
            ready: None,
            message: None,
            ping: None,
            pong: None,
            close: None,
            error: None,
        }
    }

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

    pub fn write(&mut self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }


    pub fn write_json<J>(&mut self, object: &J) -> Result<()>
    where 
        J: ?Sized + Serialize
    {
        Ok(())
    }

    pub fn write_baniry(&mut self, data: Vec<u8>) -> Result<()> {
        Ok(())
    }


    // fn clone(&self) -> Self {
    //     return Ws {
    //         rw: self.rw, 
    //         ready: self.ready,
    //         message: self.message,
    //         ping: self.ping,
    //         pong: self.pong,
    //         close: self.close,
    //         error: self.error,
    //     }
    // }
}




pub(crate) struct WsWrite<'a, RW> {
    pub(crate) writer: SplitSink<WebSocketStream<Pin<&'a mut BufReader<RW>>>, Message>
}


impl <'a, RW>WsWrite<'a, RW> {
    pub fn write(&mut self, payload: Vec<u8>) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        // let _ = self.writer.send(Message::Text(String::from_utf8(payload).unwrap().into())).await;

        Ok(())
    }
}