use std::{
    io::Result, mem::take, pin::Pin, sync::Arc
};

use futures_util::{stream::{SplitSink, SplitStream}, FutureExt, SinkExt, StreamExt};
use tokio::sync::Mutex;
use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_tungstenite::WebSocketStream;
use tracing::event;
use tungstenite::Message;
use tungstenite::protocol::Role::Server;

use crate::{
    request::Request,
    response::{new_response, parse, Response},
    ws::{Event, Reason, Ws},
    HTTP
};

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

pub(crate) struct Handler<'a, R> {
    pub(crate) http: &'a mut HTTP,
    pub(crate) writer: Pin<&'a mut BufReader<R>>,
    pub(crate) req: Request,
    pub(crate) res: Response,

}




impl <'a, R>Handler<'a, R>
where
    R: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a
{
    pub fn new(http: &'a mut HTTP, writer: Pin<&'a mut BufReader<R>>, req: Request) -> Self {
        return Self{
            http: http,
            writer: writer,
            req: req,
            res: new_response()
        };
    }

    pub async fn handle(&'a mut self) -> Result<()> {
        self.handshake().await.unwrap();

        let ws_stream = WebSocketStream::from_raw_socket(self.writer.as_mut(), Server, None)
            .await;
        let (sink, stream) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sink = Arc::new(Mutex::new(sink));

        
        self.res.ws = Some(Ws::new(tx));

        self.http
            .router()
            .router
            .match_ws_routes(&mut self.req, &mut self.res)
            .await
            .unwrap();

        if let Some(ws) = self.http.router().router.match_ws_routes(&mut self.req, &mut self.res).await {



            while let Some(message) = rx.recv().await {

                println!("Message {}", message.to_text().unwrap())
            }
            


            // TODO: live time error
            // self.listen(&mut self.res.ws.unwrap(), stream).await
        }

        
      

        return Ok(())
    }

    async fn handshake(&mut self) -> Result<()> {
        let sec_websocket_key = self.get_sec_web_socket_accept(self.req.header("sec-websocket-key"));

        self.res.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), sec_websocket_key);

        self.writer
            .write(parse(&mut self.res)
            .unwrap().as_bytes())
            .await
            .unwrap();

        return Ok(());
    }

    fn get_sec_web_socket_accept(&mut self, key: String) -> String {
        let mut hasher = Sha1::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());

        let result = hasher.finish();
        
        // TODO: use the new implementation...
        return base64::encode(&result)
    }

    async fn listen(&mut self, ws: &'a mut Ws, mut stream: SplitStream<WebSocketStream<Pin<&'a mut BufReader<R>>>>)
    where
        R: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        while let Some(msg) = stream.next().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    match msg.unwrap() {
                        Message::Text(data) => {
                            if ws.event.is_some() {
                                println!("MESSAGE IN EVEN LOOP ---> {}", data.to_string());
                                // TODO: must pass ws as or not...
                                ws.event.as_deref().unwrap()(Event::Message(data.as_bytes().to_vec())).await
                            }
                        },
                        Message::Binary(bytes) => {
                            if ws.event.is_some() {
                                ws.event.as_deref().unwrap()(Event::Message(bytes.to_vec())).await;
                            }
                        },
                        Message::Ping(bytes) => {
                            if ws.event.is_some() {
                                ws.event.as_deref().unwrap()(Event::Ping(bytes.to_vec())).await;
                            }
                        },
                        Message::Pong(bytes) => {
                            if ws.event.is_some() {
                                ws.event.as_deref().unwrap()(Event::Pong(bytes.to_vec())).await;
                            }
                        },
                        Message::Close(close_frame) => {
                            if ws.event.is_some() {
                                let callback = ws.event.as_deref().unwrap();

                                if close_frame.is_none() {
                                    return callback(Event::Close(None)).await;
                                }

                                let close = close_frame.unwrap();

                                callback(Event::Close(Some(Reason{
                                    code: close.code.into(),
                                    message: close.reason.to_string()
                                }))).await;
                            }
                        },
                        Message::Frame(_) => {/* When reading frame will not be called... */},
                    }
                });
            });
        }
    }
}