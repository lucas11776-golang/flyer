pub mod request;
pub mod handler;
pub mod response;
pub mod router;
pub mod utils;

use tokio::net::{TcpListener, TcpStream};

use std::io::{Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::{scope};


use tokio::io::{AsyncBufReadExt, BufReader};

use openssl::ssl::{
    SslAcceptor,
    SslAcceptorBuilder,
    SslFiletype,
    SslMethod
};

use crate::handler::http1x;
use crate::handler::http2x::H2_PREFACE;
use crate::router::{new_router, Router};

pub struct HTTP {
    acceptor: Option<Arc<SslAcceptor>>,
    listener: TcpListener,
    request_max_size: i64,
    router: Router,
}

pub async fn server(host: String, port: i32) -> Result<HTTP> {
    let http = HTTP {
        listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        request_max_size: 1024,
        acceptor: None,
        router: new_router(),
    };
    return Ok(http);
}

pub async fn server_tls(host: String, port: i32, key: String, certs: String) -> Result<HTTP> {
    let mut acceptor: SslAcceptorBuilder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    
    acceptor.set_private_key_file(key, SslFiletype::PEM)?;
    acceptor.set_certificate_chain_file(certs)?;
    acceptor.check_private_key()?;

    let http = HTTP {
        acceptor: Some(Arc::new(acceptor.build())),
        listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        request_max_size: 1024,
        router: new_router(),
    };

    return Ok(http);
}

impl HTTP {
    pub fn host(&self) -> String {
        return self.listener.local_addr().unwrap().ip().to_string();
    }

    pub fn port(&self) -> i32 {
        return self.listener.local_addr().unwrap().port().into();
    }

    pub fn address(&self) -> String {
        return std::format!("{0}:{1}", self.host(), self.port());
    }

    pub fn set_request_max_size(&mut self, size: i64) {
        self.request_max_size = size;    
    }

    async fn handle_stream(&mut self, stream: TcpStream, addr:  SocketAddr) -> Result<()> {
        let mut reader = BufReader::new(stream);
        let buf = reader.fill_buf().await?;


        match buf.len() >= H2_PREFACE.len() && &buf[..H2_PREFACE.len()] == H2_PREFACE {
            true => {

            },
            false => http1x::handle(self,reader, addr).await?,
        }

        Ok(())
    }

    pub async fn listen(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    scope(|_| {
                        match &self.acceptor {
                            Some(acceptor) => {
                                // Must add tokio openssl does not support direct openssl...
                            },
                            None => {
                                tokio_scoped::scope(|scope| {
                                    scope.spawn(async {
                                        match self.handle_stream(stream, addr).await {
                                            Ok(_) => {},
                                            Err(err) => println!("{}", err),
                                        }
                                    });
                                });
                            },
                        }
                    });
                },
                Err(err) => println!("{}", err),
            }
        }
    }

    pub fn router(&mut self) -> &mut Router {
        return &mut self.router;
    }
}