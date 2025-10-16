use std::{
    io::Result,
    net::SocketAddr,
    sync::Arc
};
use std::pin::{pin};

use tokio::net::{TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};



use futures::future::{BoxFuture, Future, FutureExt};

use crate::request::Request;
use crate::response::Response;
use crate::router::{Route, WebRoute, WsRoute};
use crate::server::handler::http1::NewHandler;
// use crate::server::handler::http1::{Handler as Http1Handler, NewHandler};
use crate::server::handler::http2::Handler as Http2Handler;
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{get_server_config, Protocol, HttpRequestCallback, HTTP1, HTTP2};
use crate::HTTP;




pub struct NewTcpServer {
    http_1_handler: NewHandler,
    // http_2_handler: Http2Handler,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    callback: Option<Box<HttpRequestCallback>>,
}


impl <'a>NewTcpServer {
    pub async fn new(host: String, port: i32, tls: Option<TlsAcceptor>) -> Result<NewTcpServer> {
        return Ok(NewTcpServer{
            http_1_handler: NewHandler::new(),
            listener: TcpListener::bind(format!("{}:{}", host, port)).await.unwrap(),
            acceptor: tls,
            callback: None
        });
    }

    // pub fn on_request<C, F>(&mut self, callback: C)
    // where
    //     C: FnOnce(Request, Response) -> F + Send + Sync + 'static,
    //     F: Future<Output = ()> + Send + Sync + 'static
    // {
    //     // self.callback = Some(Box::new(move |req: Request, res: Response| {
    //     //     return callback(req, res).boxed();
    //     // }));
    // }



    pub async fn http_request<C, F>(&mut self, future: C) -> & mut Self
    where
        C: FnOnce(&'a mut Request, &'a mut Response) -> F + Send + Sync + 'a,
        F: Future<Output = &'a mut Response> + Send + 'a,
    {



        return self;
    }


    pub async fn ws_request<'s, C, F>(&'s mut self, future: C) -> &mut Self
    where
        C: FnOnce(&'s mut Request, &'s mut Response) -> F + Send + Sync + 'a,
        F: Future<Output = Option<&'a mut Route<WsRoute>>> + Send + 'a,
        'a: 's
    {

        return self;
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

    fn connection_protocol(&'_ mut self, buffer: &[u8]) -> Protocol
    {
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => HTTP2,
            false => HTTP1,
        }
    }

    async fn handle_connection<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        if self.callback.is_none() {
            return Ok(())
        }

        match self.connection_protocol(rw.fill_buf().await?) {
            HTTP2 => {}
            _ => self.http_1_handler.handle(self.callback.as_ref().unwrap(), rw, addr).await.unwrap()
        }

        Ok(())
    }
}


pub struct TcpServer {
    // http_1_handler: Http1Handler,
    http_2_handler: Http2Handler,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    // http: &'a mut HTTP<'a>,
    callback: Option<Box<HttpRequestCallback>>,
}

impl <'a>TcpServer {
    // pub async fn new(http: &'a mut HTTP<'a>) -> TcpServer<'a> {
    //     let mut acceptor: Option<TlsAcceptor> = None;

    //     if http.tls.is_some() {
    //         acceptor = Some(TlsAcceptor::from(Arc::new(get_server_config(&http.tls.as_ref().unwrap()).unwrap())))
    //     }

    //     return Self {
    //         listener: TcpListener::bind(http.address()).await.unwrap(),
    //         http_1_handler: Http1Handler::new(),
    //         http_2_handler: Http2Handler::new(),
    //         acceptor: acceptor,
    //         http: http,
    //         callback: None
    //     };
    // }


    // pub async fn on_request(&'a mut self, callback: Box<RequestCallback>) {
    //     self.callback = Some(callback);
    // }

    // pub async fn listen(&mut self) {
    //     loop {
    //         match self.listener.accept().await {
    //             Ok((stream, addr)) => {
    //                 tokio_scoped::scope(|scope| {
    //                     scope.spawn(self.new_connection(stream, addr));
    //                 });
    //             },
    //             Err(_) => {}, // TODO: Log
    //         }
    //     }
    // }

    // async fn new_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    // where
    //     RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    // {
    //     match &self.acceptor {
    //         Some(acceptor) => {
    //             let _ = match acceptor.accept(stream).await {
    //                 Ok(stream) => self.handle_stream(pin!(BufReader::new(stream)), addr).await.unwrap(),
    //                 Err(_) => {}, // TODO: Log
    //             };
    //         },
    //         None => self.handle_stream(pin!(BufReader::new(stream)), addr).await.unwrap(),
    //     };
    // }

    // fn get_protocol(&'_ mut self, buffer: &[u8]) -> Protocol
    // {
    //     match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
    //         true => HTTP2,
    //         false => HTTP1,
    //     }
    // }

    // async fn handle_stream<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    // where
    //     RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    // {

    //     Ok(())

    //     // Ok(
    //     //     match self.get_protocol(rw.fill_buf().await?) {
    //     //         HTTP2 => self.http_2_handler.handle(self.http, rw, addr).await.unwrap(),
    //     //         _ => self.http_1_handler.handle(self.http, rw, addr).await.unwrap()
    //     //     }
    //     // )
    // }
}


