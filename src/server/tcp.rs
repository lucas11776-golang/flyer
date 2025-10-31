use std::{
    io::Result,
    net::SocketAddr,
};
use std::pin::{pin, Pin};

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::response::Response;
use crate::server::handler::{http1, http2, ws_http1};
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{Protocol, HTTP1, HTTP2};
use crate::HTTP;

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


    async fn http_1_protocol<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);

        let req = handler.handle().await.unwrap().unwrap();


        if req.header("upgrade") == "websocket" {
            return Ok(ws_http1::Handler::new(rw).handle(req).await.unwrap())
        }

        let res = self.http.router.match_web_routes(req, Response::new()).await.unwrap();
    
        handler.write(&mut self.http.render_response_view(res)).await.unwrap();

        Ok(())
    }

    async fn http_2_protocol<RW>(&mut self, rw: std::pin::Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = http2::Handler::new(addr, rw).await;

        while let Some(result) = handler.handle().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let (request, send) = result.unwrap();
                    let req = handler.get_http_request(request).await.unwrap();
                    let res = self.http.router.match_web_routes(req, Response::new()).await.unwrap();

                    handler.write(send,&mut self.http.render_response_view(res)).await.unwrap();
                });
            });
        }

        Ok(())   
    }

    async fn handle_connection<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
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