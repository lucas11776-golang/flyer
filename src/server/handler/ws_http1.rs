use std::{io::Result};
use std::pin::Pin;

use base64::Engine;
use base64::engine::general_purpose;
use bytes::Bytes;
use futures::future::join;
use futures::{SinkExt, StreamExt};

use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::WebSocketStream;

use tungstenite::Utf8Bytes;
use tungstenite::{Message, protocol::Role::Server};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::unbounded_channel;

use crate::router::route::Route;
use crate::ws::Reason;
use crate::{
    request::Request,
    response::{Response, parse},
    router::WsRoute,
    ws::{Event, SEC_WEB_SOCKET_ACCEPT_STATIC, Writer as WriterInterface, Ws}
};

pub(crate) enum Type {
    Close,
    Binary,
    Text,
    Ping,
    Pong,
}

pub(crate) struct Payload {
    pub method: Type,
    pub data: Vec<u8>
}

pub struct Writer {
    pub(crate) sender: UnboundedSender<Payload>
}

pub(crate) struct Handler<'a ,RW> {
    pub sink: futures::stream::SplitSink<WebSocketStream<Pin<&'a mut BufReader<RW>>>, Message>,
    pub stream: futures::stream::SplitStream<WebSocketStream<Pin<&'a mut BufReader<RW>>>>,
    pub receiver: UnboundedReceiver<Payload>,
    pub ws: Ws
}

impl WriterInterface for Writer {
    fn write(&mut self, data: Vec<u8>) {
        self.sender.send(Payload { method: Type::Text, data: data }).unwrap();
    }

    fn write_binary(&mut self, data: Vec<u8>) {
        self.sender.send(Payload { method: Type::Binary, data: data }).unwrap();
    }

    fn ping(&mut self, data: Vec<u8>) {
        self.sender.send(Payload { method: Type::Ping, data: data }).unwrap();
    }

    fn pong(&mut self, data: Vec<u8>) {
        self.sender.send(Payload { method: Type::Pong, data: data }).unwrap();
    }
    
    fn close(&mut self) {
        self.sender.send(Payload { method: Type::Close, data: vec![] }).unwrap()
    }
}

impl <'a, RW>Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    pub async fn new(rw: Pin<&'a mut BufReader<RW>>, req: &'a mut Request, res: &'a mut Response) -> Result<(Self, &'a mut Request, &'a mut Response)> {
        let (rw, req, res) = Self::handshake(rw, req, res).await.unwrap();
        let (sink, stream) = WebSocketStream::from_raw_socket(rw, Server, None).await.split();
        let (tx, rx) = unbounded_channel::<Payload>();
        res.ws = Some(Box::new(Writer{sender: tx}));

        return Ok((Self {
            sink: sink,
            stream: stream,
            receiver: rx,
            ws: Ws::new(),
        }, req, res));
    }

    pub async fn handle(&mut self, route: &'a mut Route<Box<WsRoute>>, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        let receiver = async {
            while let Some(payload) = self.receiver.recv().await {
                match payload.method {
                    Type::Binary => self.sink.send(Message::Binary(Bytes::from(payload.data))).await.unwrap(),
                    Type::Text => self.sink.send(Message::Text(Utf8Bytes::from(String::from_utf8(payload.data).unwrap()))).await.unwrap(),
                    Type::Ping => self.sink.send(Message::Ping(Bytes::from(payload.data))).await.unwrap(),
                    Type::Pong => self.sink.send(Message::Pong(Bytes::from(payload.data))).await.unwrap(),
                    Type::Close => {
                        let s = self.sink.send(Message::Close(None)).await;

                        if s.is_ok() {
                            s.unwrap();
                        }

                        break;
                    },
                }
            }

            self.sink.close().await.unwrap();
        };

        let stream = async {
            (route.route)(req, &mut self.ws);

            let writer = res.ws.as_mut().unwrap();

            while let Some(message) = self.stream.next().await {
                let message = message.unwrap();

                if self.ws.event.is_none() {
                    continue;
                }
                
                match message {
                    Message::Text(data) => {
                        self.ws.event.as_mut().unwrap()(Event::Text(data.as_bytes().to_vec()), writer);
                    },
                    Message::Binary(data) => {
                        self.ws.event.as_mut().unwrap()(Event::Binary(data.to_vec()), writer);
                    },
                    Message::Ping(data) => {
                        self.ws.event.as_mut().unwrap()(Event::Ping(data.to_vec()), writer);
                    },
                    Message::Pong(data) => {
                        self.ws.event.as_mut().unwrap()(Event::Pong(data.to_vec()), writer);
                    },
                    Message::Close(close_frame) => {
                        let callback = self.ws.event.as_deref().unwrap();

                        if close_frame.is_none() {
                            return callback(Event::Close(None), writer);
                        }

                        let close = close_frame.unwrap();

                        callback(Event::Close(Some(Reason{
                            code: close.code.into(),
                            message: close.reason.to_string()
                        })), writer);
                    },
                    Message::Frame(_) => {/* When reading frame will not be called... */},
                }
            }

            writer.close();
        };

        join(receiver, stream).await;

        Ok(())
    }

    async fn handshake(mut rw: Pin<&'a mut BufReader<RW>>, req: &'a mut Request, res: &'a mut Response) -> Result<(Pin<&'a mut BufReader<RW>>, &'a mut Request, &'a mut Response)> {
        let res = res.status_code(101)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Accept", Self::get_sec_web_socket_accept(req.header("sec-websocket-key")).as_str());

        rw.as_mut()
            .write(parse(res, Some(&mut req.cookies.new_cookie)).unwrap().as_bytes())
            .await
            .unwrap();

        return Ok((rw, req, res));
    }

    fn get_sec_web_socket_accept(key: String) -> String {
        let mut hasher = Sha1::new();
        let mut accept = String::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        general_purpose::STANDARD.encode_string(hasher.finish(), &mut accept);
        
        return accept;
    }
}


