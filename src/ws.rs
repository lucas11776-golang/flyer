
use std::cell::UnsafeCell;
use std::future::Future;
use std::{io::{Error, Result}, pin::Pin};

use bytes::Bytes;
use event_emitter_rs::EventEmitter;
use futures_util::future::BoxFuture;
use futures_util::StreamExt;
use futures_util::{stream::SplitSink, SinkExt};
use serde::Serialize;
use tokio::{io::{AsyncRead, AsyncWrite, BufReader}};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Message, Utf8Bytes};

use crate::request::Request;
use crate::response::new_response;
use crate::HTTP;

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub type OnReady = fn (ws: &mut Ws);
pub type OnMessage = fn (data: Vec<u8>);
pub type OnPing = fn (data: Vec<u8>);
pub type OnPong = fn (data: Vec<u8>);
pub type OnClose = fn (code: u16);
pub type OnError = fn (error: Error);

pub trait Rw<'a, R>: Send + Sync
{
    fn new(http: &'a mut HTTP, req: &'a mut Request, client: WebSocketStream<Pin<&'a mut BufReader<R>>>) -> Self;
    fn send(&mut self, item: Message) -> impl Future<Output = Result<()>>;
    fn listen(&mut self) -> impl Future<Output = Result<()>>;
    fn close() -> impl Future<Output = Result<()>>;
}

pub struct Client<'a, R> {
    // pub(crate) ws: Ws,
    pub(crate) req: &'a mut Request,
    pub(crate) http: &'a mut HTTP,
    pub(crate) read: futures_util::stream::SplitStream<WebSocketStream<Pin<&'a mut BufReader<R>>>>,
    pub(crate) write: SplitSink<WebSocketStream<Pin<&'a mut BufReader<R>>>, Message>,
}

impl <'a, R> Rw<'a, R> for Client<'a, R>
where
    R: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    fn new(http: &'a mut HTTP, req: &'a mut Request, client: WebSocketStream<Pin<&'a mut BufReader<R>>>) -> Client<'a, R> {
        let (write, read) = client.split();

        return Client {
            req: req,
            http: http,
            read: read,
            write: write
        }
    }

    async fn send(&mut self, item: Message) -> Result<()> {
        todo!()
    }
    
    async fn listen(&mut self) -> Result<()> {
        let mut res = new_response();

        res.ws = Some(Ws {
            emitter: EventEmitter::new(),
            ready: None,
            message: None,
            ping: None,
            pong: None,
            close: None,
            error: None
        });

        match self.http.router.match_ws_routes(self.req, &mut res) {
            Some(_) => {
                while let Some(message) = self.read.next().await {
                    match message {
                        Ok(msg) => {
                            match msg {
                                Message::Text(text) => {
                                    println!("Received text: {}", text);
                                }
                                Message::Binary(bin) => {
                                    println!("Received binary data: {:?}", bin);
                                }
                                Message::Ping(bin) => {
                                    println!("Received ping data: {:?}", bin);
                                }
                                Message::Pong(bin) => {
                                    println!("Received pong data: {:?}", bin);
                                }
                                _ => {
                                    println!("Received other message type");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error receiving message: {}", e);
                            break;
                        }
                    }
                }
            },
            None => todo!(),
        }

        Ok(())
    }
    
    async fn close() -> Result<()> {
        todo!()
    }
}



pub struct Ws
{
    pub(crate) emitter: event_emitter_rs::EventEmitter,
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