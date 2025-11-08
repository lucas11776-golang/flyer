use std::{io::Result};
use std::pin::Pin;
use std::pin::{pin};

use event_emitter_rs::Listener;
use futures::future::join;
use futures::{SinkExt, StreamExt};
use futures::join;
use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Utf8Bytes;
use tungstenite::{Message, protocol::Role::Server};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::router::group::GroupRouter;
use crate::{
    request::Request,
    response::{Response, parse},
    router::{Route, WsRoute},
    ws::{Event, SEC_WEB_SOCKET_ACCEPT_STATIC, Writer as WriterInterface, Ws}
};


pub struct Writer {
    pub(crate) sender: UnboundedSender<Payload>
}

pub(crate) struct Handler {
    pub receiver: UnboundedReceiver<Payload>
}

impl WriterInterface for Writer {
    fn write(&mut self, data: Vec<u8>) {
        self.sender.send(Payload { method: PayloadType::Text, data: data }).unwrap()
    }

    fn write_binary(&mut self, data: Vec<u8>) {
        println!("Writer Binary");
    }
    
    fn close(&mut self) {
        self.sender.send(Payload { method: PayloadType::Close, data: vec![] }).unwrap()
    }
}

pub(crate) enum PayloadType {
    Close,
    Binary,
    Text,
    Ping,
    Pong,
}

pub(crate) struct Payload {
    pub method: PayloadType,
    pub data: Vec<u8>
}

impl <'a>Handler
{
    pub fn new(receiver: UnboundedReceiver<Payload>,) -> Self {
        return Self {
            receiver: receiver
        }
    }

    pub async fn handle(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        // let (req, res) = self.handshake(req, res).await.unwrap();
        // let ws_stream = WebSocketStream::from_raw_socket(self.rw.as_mut(), Server, None).await;
        // let (mut sink, mut stream) = ws_stream.split();
        // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Payload>();
        // let ws = Ws::new();
        // let writer = Writer{sender: tx};

        // let messages = async {
        //     println!(" ************************************ RUNNING MESSAGE ************************************ ");

        //     while let Some(payload) = rx.recv().await {
        //         match payload.method {
        //             PayloadType::Binary => todo!(),
        //             PayloadType::Text => sink.send(Message::Text(Utf8Bytes::from(String::from_utf8(payload.data).unwrap()))).await.unwrap(),
        //             PayloadType::Ping => todo!(),
        //             PayloadType::Pong => todo!(),
        //             PayloadType::Close => {
        //                 break;
        //             },
        //         }
        //     }

        //     sink.close().await.unwrap();
        // };

        // let listener = async {
        //     println!(" ************************************ RUNNING listener ************************************ ");

        //     /***********************************************************************************************
        //      TODO: handle middleware and put writer in ws as trait to support (http3 websocket read more)
        //     ***********************************************************************************************/
        //     res.ws = Some((ws, Box::new(writer))); // Ready for middleware

        //     let (ws, writer) = res.ws.as_mut().unwrap();

        //     (route.route)(req, ws);

        //     while let Some(message) = stream.next().await {
        //         match message.unwrap() {
        //             Message::Text(data) => ws.event.as_mut().unwrap()(Event::Message(data.as_bytes().to_vec()), writer),
        //             Message::Binary(bytes) => {
        //             },
        //             Message::Ping(bytes) => {
        //             },
        //             Message::Pong(bytes) => {
        //             },
        //             Message::Close(close_frame) => {
        //             },
        //             Message::Frame(_) => {/* When reading frame will not be called... */},
        //         }
        //     }

        //     writer.close();
        // };


        // join(messages, listener).await;

        // println!(" ************************************ END ************************************ ");

        Ok(())
    }

    // async fn handshake(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
    //     let res = res.status_code(101)
    //         .header("Upgrade".to_owned(), "websocket".to_owned())
    //         .header("Connection".to_owned(), "Upgrade".to_owned())
    //         .header("Sec-WebSocket-Accept".to_owned(), self.get_sec_web_socket_accept(req.header("sec-websocket-key")));

    //     self.rw
    //         .as_mut()
    //         .write(parse(res).unwrap().as_bytes())
    //         .await
    //         .unwrap();

    //     return Ok((req, res));
    // }

    // fn get_sec_web_socket_accept(&mut self, key: String) -> String {
    //     let mut hasher = Sha1::new();
        
    //     hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        
    //     // TODO: use the new implementation...
    //     return base64::encode(&hasher.finish())
    // }
}


