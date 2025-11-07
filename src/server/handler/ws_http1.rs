use std::io::Result;
use std::pin::Pin;

use futures::StreamExt;
use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{Message, protocol::Role::Server};

use crate::{
    request::Request,
    response::{Response, parse},
    router::{Route, WsRoute},
    ws::{Event, SEC_WEB_SOCKET_ACCEPT_STATIC, Writer as WriterInterface, Ws}
};


pub struct Writer {}

pub(crate) struct Handler<'a, RW> {
    rw: Pin<&'a mut BufReader<RW>>,
}

impl WriterInterface for Writer
{
    fn write(&mut self, data: Vec<u8>) {
        println!("Writer Text");
    }

    fn write_binary(&mut self, data: Vec<u8>) {
        println!("Writer Binary");
    }
}

impl <'a, RW>Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    pub fn new(rw: Pin<&'a mut BufReader<RW>>) -> Handler<'a, RW> {
        return Self {
            rw: rw
        }
    }

    pub async fn handle(&mut self, route: &'a mut Route<Box<WsRoute>>, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        let (req, res) = self.handshake(req, res).await.unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(self.rw.as_mut(), Server, None).await;
        let (mut _sink, mut stream) = ws_stream.split();
        let ws = Ws::new();
        let writer = Writer{}; // TODO: 

        /***********************************************************************************************
          TODO: handle middleware and put writer in ws as trait to support (http3 websocket read more)
        ***********************************************************************************************/
        res.ws = Some((ws, Box::new(writer))); // Ready for middleware

        let (ws, writer) = res.ws.as_mut().unwrap();

        (route.route)(req, ws);

        while let Some(message) = stream.next().await {
            match message.unwrap() {
                Message::Text(data) => {
                    ws.event.as_mut().unwrap()(Event::Message(data.as_bytes().to_vec()), writer)
                },
                Message::Binary(bytes) => {
                },
                Message::Ping(bytes) => {

                },
                Message::Pong(bytes) => {
                },
                Message::Close(close_frame) => {
                },
                Message::Frame(_) => {/* When reading frame will not be called... */},
            }
        }

        Ok(())
    }

    async fn handshake(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        let res = res.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), self.get_sec_web_socket_accept(req.header("sec-websocket-key")));

        self.rw
            .as_mut()
            .write(parse(res).unwrap().as_bytes())
            .await
            .unwrap();

        return Ok((req, res));
    }

    fn get_sec_web_socket_accept(&mut self, key: String) -> String {
        let mut hasher = Sha1::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        
        // TODO: use the new implementation...
        return base64::encode(&hasher.finish())
    }
}


