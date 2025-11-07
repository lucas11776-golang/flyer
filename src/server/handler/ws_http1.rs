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
    ws::{Event, SEC_WEB_SOCKET_ACCEPT_STATIC, Writer, Ws}
};

pub(crate) struct Handler<'a, RW> {
    rw: Pin<&'a mut BufReader<RW>>,
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
        println!("Handle HTTP1.1 Protocol Websocket");

        let req = self.handshake(req, res).await.unwrap();

        let ws_stream = WebSocketStream::from_raw_socket(self.rw.as_mut(), Server, None)
            .await;
        let (mut sink, mut stream) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut ws = Ws::new();
        let mut writer = Writer::new(tx);

        /***********************************************************************************************
          TODO: handle middleware and put writer in ws as trait to support (http3 websocket read more)
        ***********************************************************************************************/

        (route.route)(req, &mut ws);

        while let Some(message) = stream.next().await {
            match message.unwrap() {
                Message::Text(data) => {
                    ws.event.as_mut().unwrap()(Event::Message(data.as_bytes().to_vec()), &mut writer)
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

    async fn websocket_event(&mut self) -> Result<()> {


        Ok(())
    }

    async fn handshake(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<&'a mut Request> {
        let res = res.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), self.get_sec_web_socket_accept(req.header("sec-websocket-key")));

        self.rw
            .as_mut()
            .write(parse(res).unwrap().as_bytes())
            .await
            .unwrap();

        return Ok(req);
    }

    fn get_sec_web_socket_accept(&mut self, key: String) -> String {
        let mut hasher = Sha1::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        
        // TODO: use the new implementation...
        return base64::encode(&hasher.finish())
    }
}


