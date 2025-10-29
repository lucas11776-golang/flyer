use std::{
    io::Result,
    net::SocketAddr,
    sync::Arc
};
use std::pin::{pin, Pin};

use tokio::net::{TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};



use futures::future::{BoxFuture, Future, FutureExt};

use crate::request::Request;
use crate::response::{new_response, Response};
use crate::router::{Route, WebRoute, WsRoute};
use crate::server::handler::http1::NewHandler;
// use crate::server::handler::http1::{Handler as Http1Handler, NewHandler};
use crate::server::handler::http2::Handler as Http2Handler;
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{get_server_config, Protocol, HttpRequestCallback, HTTP1, HTTP2};
use crate::HTTP;




pub struct NewTcpServer<'a> {
    // http_1_handler: Option<NewHandler<'a, RW>>,
    // http_2_handler: Http2Handler,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    // callback: Option<Box<HttpRequestCallback>>,
    http: &'a mut HTTP
}


impl <'a>NewTcpServer<'a> {
    // pub async fn new(host: String, port: i32, tls: Option<TlsAcceptor>) -> Result<NewTcpServer> {
    //     return Ok(NewTcpServer{
    //         // http_1_handler: NewHandler::new(),
    //         listener: TcpListener::bind(format!("{}:{}", host, port)).await.unwrap(),
    //         acceptor: tls,
    //         callback: None
    //     });
    // }

    pub async fn new(http: &'a mut HTTP) -> Result<NewTcpServer<'a>> {
        return Ok(NewTcpServer{
            listener: TcpListener::bind(format!("{}", http.address())).await.unwrap(),
            acceptor: http.get_tls_acceptor().unwrap(),
            http: http,
        });
        // return new_tcp_server;
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



    pub async fn http_request<C, F>(&'a mut self, callback: C) -> & mut Self
    where
        C: FnOnce(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static
    {
        // self.callback = Some(Box::new(move |req: Request, res: Response| callback(req, res).boxed()));

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

    fn new_response(&mut self) -> Response {
        let mut res = new_response(); 

        // if let Some(path) = self.http.configuration.get("view_path") {
        //     res.view = Some(new_view(path.to_string()))
        // }

        return res;
    }

    // TODO: refactor to best module.
    fn render_view(&mut self, mut res: Response) -> Response {
        return match res.view  {
            Some(bag) => {
                match self.http.view.as_mut() {
                    Some(view) => {
                        res.body =  view.render(&bag.view, bag.data).as_bytes().to_vec();
                    },
                    None => {
                        res.status_code = 500;
                        println!("Set View Path") // TODO: log
                    },
                }

                res.view = None;

                res
            },
            None => {
                res
            },
        };

    }

    async fn http_1_protocol<RW>(&mut self,  mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = NewHandler::new(Pin::new(&mut rw), addr);

        let req = handler.handle()
            .await
            .unwrap()
            .unwrap();
        let res = self.new_response();
        let res = self.http.router.match_web_routes(req, res).await.unwrap();
    
        handler.write(&mut self.render_view(res)).await.unwrap();

        Ok(())
    }



    async fn handle_connection<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match self.connection_protocol(rw.fill_buf().await?) {
            HTTP2 => {}
            _ => self.http_1_protocol(rw, addr).await.unwrap()
        }

        Ok(())
    }
}