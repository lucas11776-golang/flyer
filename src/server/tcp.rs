use std::fmt::Debug;
use std::{
    io::Result,
    net::SocketAddr,
    sync::Arc
};
use std::pin::{pin};

use tokio::net::{TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::server::handler::{http1, http2};
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{get_server_config, Protocol, HTTP1, HTTP2};
use crate::HTTP;

pub struct TcpServer<'a> {
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    http: &'a mut HTTP
}

impl <'a>TcpServer<'a> {
    pub async fn new(http: &'a mut HTTP) -> TcpServer<'a> {
        match &http.tls {
            Some(tls) => TcpServer {
                listener: TcpListener::bind(http.address()).await.unwrap(),
                acceptor: Some(TlsAcceptor::from(Arc::new(get_server_config(tls).unwrap()))),
                http: http,
            },
            None => TcpServer {
                listener: TcpListener::bind(http.address()).await.unwrap(),
                acceptor: None,
                http: http,
            },
        }
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
        RW: AsyncRead + AsyncWrite + Unpin + Send + Debug + Sync
    {
        match &self.acceptor {
            Some(acceptor) => {
                let _ = match acceptor.accept(stream).await {
                    Ok(stream) => self.handle_connection(stream, addr).await,
                    Err(_) => {}, // TODO: Log
                };
            },
            None => self.handle_connection(stream, addr).await,
        };
    }

    async fn handle_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Debug + Sync
    {
        match self.handle_stream(pin!(BufReader::new(stream)), addr).await {
            Ok(_) => {},
            Err(_) => {}, // TODO: Log
        }
    }

    fn get_protocol(&mut self, buffer: &[u8]) -> Protocol
    {
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => HTTP2,
            false => HTTP1,
        }
    }

    async fn handle_stream<RW>(&mut self, mut rw: std::pin::Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Debug + Sync
    {
        Ok(
            match self.get_protocol(rw.fill_buf().await?) {
                HTTP2 => http2::Handler::handle(self.http, rw, addr).await.unwrap(),
                _ => http1::Handler::handle(self.http, rw, addr).await.unwrap() // TODO: bad must check if is HTTP1 or HTTP2 or drop
            }
        )
    }

}


