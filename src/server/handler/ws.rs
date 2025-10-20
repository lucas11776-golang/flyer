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
    // pub(crate) http: &'a mut HTTP<'a>,
    pub(crate) writer: Pin<&'a mut BufReader<R>>,
}

impl <'a, R>Handler<'a, R>
where
    R: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'a
{
    // pub fn new(http: &'a mut HTTP<'a>, writer: Pin<&'a mut BufReader<R>>) -> Self {
    //     return Self{
    //         http: http,
    //         writer: writer,
    //     };
    // }

    fn websocket_handle_shake_headers(&mut self, sec_websocket_key: String, res: Response) -> Response {
        // TODO: handshake...
        return res.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), self.get_sec_web_socket_accept(sec_websocket_key));
    }

    async fn handshake(&mut self, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        res = self.websocket_handle_shake_headers(req.header("sec-websocket-key"), res);


        // self.writer
        //     .write(parse(res).unwrap().as_bytes())
        //     .await
        //     .unwrap();



        Ok((req, res))
    }

    pub async fn handle(&mut self, req: Request, mut res: Response) -> Result<()> {

        // println!("{:?}", parse(res).unwrap());


        (_, res) = self.handshake(req, res).await.unwrap();

        

        let ws_stream = WebSocketStream::from_raw_socket(self.writer.as_mut(), Server, None)
            .await;
        let (mut sink, mut stream) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();


        

        res.ws = Some(Ws::new(tx));

        // if self.http.router().router.match_ws_routes(req, res).await.is_none() {
        //     return Ok(());
        // }

        // tokio_scoped::scope(|scope| {
        //     scope.spawn(async {
        


        // while let Some(item) = rx.recv().await {
        //     sink.send(item).await.unwrap();
        // }
        //     });
        // });


        // while let Some(item) = stream.next().await {
        //     println!("Message {:?}", item.unwrap().to_text())
        // }


        // stream.


                while let Some(item) = stream.next().await {
                    println!("Message {:?}", item.unwrap().to_text())
                }




        println!("QUITTING");

        return Ok(())
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