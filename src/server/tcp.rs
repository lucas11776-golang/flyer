use std::{
    io::Result,
    net::SocketAddr,
};
use std::pin::{pin, Pin};

use futures::{SinkExt, StreamExt};
use futures::future::join;
use openssl::sha::Sha1;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tungstenite::Utf8Bytes;
use tungstenite::{Message, protocol::Role::Server};
use tokio_tungstenite::WebSocketStream;

use crate::request::Request;
use crate::response::{Response, parse};
use crate::server::handler::ws_http1::{Payload, PayloadType};
use crate::server::handler::{http1, http2, ws_http1::{Writer}};
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{Protocol, HTTP1, HTTP2};
use crate::HTTP;
use crate::ws::{Event, SEC_WEB_SOCKET_ACCEPT_STATIC, Ws};

pub(crate) struct TcpServer<'a> {
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    http: &'a mut HTTP
}

impl <'a>TcpServer<'a> {
    pub async fn new(http: &'a mut HTTP) -> Result<TcpServer<'a>> {
        return Ok(TcpServer{
            listener: TcpListener::bind(format!("{}", http.address())).await.unwrap(),
            acceptor: http.get_tls_acceptor().unwrap(),
            http: http,
        });
    }

    pub async fn listen(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tokio_scoped::scope(|scope| {
                        scope.spawn(self.new_connection(stream, addr));
                    });
                },
                Err(_) => {}, // TODO: Log
            }
        }
    }

    async fn new_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match &self.acceptor {
            Some(acceptor) => {
                let _ = match acceptor.accept(stream).await {
                    Ok(stream) => self.handle_connection(pin!(BufReader::new(stream)), addr).await.unwrap(),
                    Err(_) => {}, // TODO: Log
                };
            },
            None => self.handle_connection(pin!(BufReader::new(stream)), addr).await.unwrap(),
        }
    }

    fn connection_protocol(&'_ mut self, buffer: &[u8]) -> Protocol {
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => HTTP2,
            false => HTTP1,
        }
    }

    async fn handshake<'h, RW>(&mut self, mut rw: Pin<&'h mut BufReader<RW>>, req: &'h mut Request, res: &'h mut  Response) -> Result<(Pin<&'h mut BufReader<RW>>, &'h mut Request, &'h mut Response)>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let res = res.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), self.get_sec_web_socket_accept(req.header("sec-websocket-key")));

        rw.write(parse(res).unwrap().as_bytes()).await.unwrap();

        return Ok((rw, req, res));
    }

    fn get_sec_web_socket_accept(&mut self, key: String) -> String {
        let mut hasher = Sha1::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        
        // TODO: use the new implementation...
        return base64::encode(&hasher.finish())
    }

    /***********************************************************************************************
     TODO: Next step broke everything down and refactor...
    ***********************************************************************************************/
    async fn handle_web_socket<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>, mut req: Request, mut res: Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let (rw, req, res) = self.handshake(rw, &mut req, &mut res).await.unwrap();
        let ws_stream = WebSocketStream::from_raw_socket(rw, Server, None).await;
        let (mut sink, mut stream) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Payload>();
        let mut ws = Ws::new();
        let writer = Writer{sender: tx};

        let messages = async {
            // println!(" ************************************ RUNNING MESSAGE ************************************ ");
            while let Some(payload) = rx.recv().await {
                match payload.method {
                    PayloadType::Binary => todo!(),
                    PayloadType::Text => sink.send(Message::Text(Utf8Bytes::from(String::from_utf8(payload.data).unwrap()))).await.unwrap(),
                    PayloadType::Ping => todo!(),
                    PayloadType::Pong => todo!(),
                    PayloadType::Close => {
                        break;
                    },
                }
            }
            sink.close().await.unwrap();
        };

        let listener = async {
            // println!(" ************************************ RUNNING listener ************************************ ");
            res.ws = Some(Box::new(writer));

            if !self.http.router.match_ws_routes(req, res, &mut ws).await.unwrap() {
                let writer = res.ws.as_mut().unwrap();

                writer.close();

                return;
            }

            let writer = res.ws.as_mut().unwrap();

            while let Some(message) = stream.next().await {
                match message.unwrap() {
                    Message::Text(data) => ws.event.as_mut().unwrap()(Event::Message(data.as_bytes().to_vec()), writer),
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

            writer.close();
        };

        join(messages, listener).await;

        // println!(" ************************************ END ************************************ ");

        return Ok(());
        // return Ok(ws_http1::Handler::new(rx).handle(&mut req, &mut res).await.unwrap());
    }
 
    async fn http_1_protocol<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
        let result = handler.handle().await;

        if result.is_none() {
            return Ok(());
        }

        let mut req = result.unwrap().unwrap();
        let mut res = Response::new();

        if req.header("upgrade") == "websocket" {
            return Ok(self.handle_web_socket(rw, req, res).await.unwrap())
        }

        let res = self.http.router.match_web_routes(&mut req, &mut res).await.unwrap();

        Ok(handler.write(&mut self.http.render_response_view(res)).await.unwrap())
    }

    async fn http_2_protocol<RW>(&mut self, rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = http2::Handler::new(addr, rw).await;

        while let Some(result) = handler.handle().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let (request, send) = result.unwrap();
                    let mut req = handler.get_http_request(request).await.unwrap();
                    let mut res = Response::new();
                    let res = self.http.router.match_web_routes(&mut req, &mut res).await.unwrap();

                    handler.write(send,&mut self.http.render_response_view(res)).await.unwrap();
                });
            });
        }

        Ok(())   
    }

    async fn handle_connection<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match self.connection_protocol(rw.fill_buf().await?) {
            HTTP2 => self.http_2_protocol(rw, addr).await.unwrap(),
            _ => self.http_1_protocol(rw, addr).await.unwrap()
        }

        Ok(())
    }
}