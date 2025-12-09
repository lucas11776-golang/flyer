use std::{
    io::Result,
    net::SocketAddr,
};
use std::pin::Pin;

use rustls::ServerConfig;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::http::HTTP_CONTAINER;
use crate::request::Request;
use crate::response::Response;
use crate::server::handler::{http1, http2, ws_http1};
use crate::server::handler::http2::H2_PREFACE;
use crate::server::helpers::{setup, teardown};
use crate::server::{Protocol, get_tls_acceptor};

pub(crate) struct TcpServer {
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
}

impl <'a>TcpServer {
    #[allow(static_mut_refs)]
    pub async fn new(config: Option<ServerConfig>) -> Result<TcpServer> {
        return Ok(TcpServer{
            listener: TcpListener::bind(format!("{}", unsafe { HTTP_CONTAINER.address() })).await.unwrap(),
            acceptor: config.and_then(|config| get_tls_acceptor(config)),
        });
    }

    pub async fn listen(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let acceptor = self.acceptor.clone();

                    tokio::spawn(async move {
                        Self::new_connection(stream, addr, &acceptor).await    
                    });
                },
                Err(_) => {}, // TODO: Log
            }
        }
    }

    async fn new_connection<RW>(stream: RW, addr: SocketAddr, acceptor: &Option<TlsAcceptor>)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match acceptor {
            Some(acceptor) => {
                let _ = match acceptor.accept(stream).await {
                    Ok(stream) => Self::handle_connection(BufReader::new(stream), addr).await.unwrap(),
                    Err(_) => {}, // TODO: Log
                };
            },
            None => Self::handle_connection(BufReader::new(stream), addr).await.unwrap(),
        }
    }

    async fn handle_connection<RW>(mut rw: BufReader<RW>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match Self::connection_protocol(rw.fill_buf().await?) {
            Protocol::HTTP2 => Self::http_2_protocol(rw, addr).await.unwrap(),
            _ => Self::http_1_protocol(rw, addr).await.unwrap()
        }

        Ok(())
    }

    fn connection_protocol(buffer: &[u8]) -> Protocol {
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => Protocol::HTTP2,
            false => Protocol::HTTP1,
        }
    }

    #[allow(static_mut_refs)]
    async fn handle_web_socket<RW>(rw: BufReader<RW>, req: &mut Request, res: &mut Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        unsafe {
            let (mut handler, req, res) = ws_http1::Handler::new(rw, req, res).await.unwrap();
            let result = HTTP_CONTAINER.router.ws_match(req, res).await;

            if result.is_none() {
                return Ok(())
            }

            let (route, req, res) = result.unwrap();

            return Ok(handler.handle(route, req, res).await.unwrap());
        }
    }
 
    async fn http_1_protocol<RW>(mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
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
            (req, res) = setup(req, res).await.unwrap();

            Self::handle_web_socket(rw, &mut req, &mut res).await.unwrap();

            return Ok(())
        }

        (req, res) = Self::handle(req, res).await.unwrap();

        return Ok(handler.write(&mut req, &mut res).await.unwrap());
    }

    async fn http_2_protocol<RW>(rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
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

                    (req, res) = Self::handle(req, res).await.unwrap();

                    handler.write(send, &mut req, &mut res).await.unwrap();
                });
            });
        }

        return Ok(())   
    }

    #[allow(static_mut_refs)]
    async fn handle<'h>(mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        unsafe {
            (req, res) = setup(req, res).await.unwrap();

            res.request_headers = req.headers.clone();

            let resp = HTTP_CONTAINER.router.web_match(&mut req, &mut res).await;

            if resp.is_none() && HTTP_CONTAINER.assets.is_some() {
                (req, res) = HTTP_CONTAINER.assets.as_mut().unwrap().handle(req, res).unwrap();
            }

            return Ok(teardown(req, res).await.unwrap());
        }
    }
}

