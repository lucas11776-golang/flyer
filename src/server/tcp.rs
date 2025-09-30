use std::{
    io::Result,
    net::SocketAddr,
    sync::Arc
};
use std::pin::{pin};

use tokio::net::{TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::server::handler::http1::Handler as Http1Handler;
use crate::server::handler::http2::Handler as Http2Handler;
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{get_server_config, Protocol, HTTP1, HTTP2};
use crate::HTTP;

pub struct TcpServer<'a> {
    http_1_handler: Http1Handler,
    http_2_handler: Http2Handler,
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    http: &'a mut HTTP
}

impl <'a>TcpServer<'a> {
    pub async fn new(http: &'a mut HTTP) -> TcpServer<'a> {
        let mut acceptor: Option<TlsAcceptor> = None;

        if http.tls.is_some() {
            acceptor = Some(TlsAcceptor::from(Arc::new(get_server_config(&http.tls.as_ref().unwrap()).unwrap())))
        }

        return Self {
            listener: TcpListener::bind(http.address()).await.unwrap(),
            http_1_handler: Http1Handler::new(),
            http_2_handler: Http2Handler::new(),
            acceptor: acceptor,
            http: http,
        };
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
                    Ok(stream) => self.handle_stream(pin!(BufReader::new(stream)), addr).await.unwrap(),
                    Err(_) => {}, // TODO: Log
                };
            },
            None => self.handle_stream(pin!(BufReader::new(stream)), addr).await.unwrap(),
        };
    }

    fn get_protocol(&'_ mut self, buffer: &[u8]) -> Protocol<'_>
    {
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => HTTP2,
            false => HTTP1,
        }
    }

    async fn handle_stream<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        Ok(
            match self.get_protocol(rw.fill_buf().await?) {
                HTTP2 => self.http_2_handler.handle(self.http, rw, addr).await.unwrap(),
                _ => self.http_1_handler.handle(self.http, rw, addr).await.unwrap()
            }
        )
    }
}


