use std::{
    io::Result,
    net::SocketAddr,
    sync::Arc
};
use std::pin::{pin};

use tokio::net::{TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

// use crate::handler::http1;
// use crate::handler::http2::{H2_PREFACE};
use crate::router::GroupRouter;
use crate::server::handler::http1;
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{get_server_config, HttpConfig, RoutesCallback, Tls, WebCallback};
use crate::utils::Configuration;
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
                acceptor: Some(TlsAcceptor::from(Arc::new(get_server_config(tls.key_path.as_str(), tls.cert_path.as_str()).unwrap()))),
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
                Err(err) => println!("{}", err), // TODO: Log
            }
        }
    }

    async fn new_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        match &self.acceptor {
            Some(acceptor) => {
                let _ = match acceptor.accept(stream).await {
                    Ok(stream) => self.handle_connection(stream, addr).await,
                    Err(err) => println!("error: {}", err),
                };
            },
            None => self.handle_connection(stream, addr).await,
        };
    }

    async fn handle_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        match self.handle_stream(stream, addr).await {
            Ok(_) => {},
            Err(err) => println!("error: {}", err),
        }
    }

    async fn handle_stream<RW>(&mut self, stream: RW, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        let mut rw = pin!(BufReader::new(stream));
        let buffer = rw.fill_buf().await?;

        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => {},
            false => http1::Handler::handle(self.http, rw, addr).await.unwrap(),
        }

        Ok(())
    }

}


