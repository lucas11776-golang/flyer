use std::io::{Error, Result};

use serde::Serialize;

pub const SecWebSocketAcceptStatic: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub trait RW {
    fn read(&mut self) -> Result<Vec<u8>>;
    fn write(&mut self, payload: Vec<u8>) -> Result<()>;
}


pub type OnReady = fn (ws: &mut Ws);
pub type OnMessage = fn (data: Vec<u8>);
pub type OnPing = fn (data: Vec<u8>);
pub type OnPong = fn (data: Vec<u8>);
pub type OnClose = fn (code: u16);
pub type OnError = fn (error: Error);

pub struct Ws {
    rw: Box<dyn RW>,
    pub(crate) ready: Option<OnReady>,
    pub(crate) message: Option<OnMessage>,
    pub(crate) ping: Option<OnPing>,
    pub(crate) pong: Option<OnPong>,
    pub(crate) close: Option<OnClose>,
    pub(crate) error: Option<OnError>,
}

impl Ws {
    pub fn new(rw: Box<dyn RW>) -> Self {
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

    pub(crate) fn listen(&mut self) -> Result<()> {
        for data  in self.rw.read().unwrap() {

        }

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
}
