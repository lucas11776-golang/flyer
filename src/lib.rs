pub mod request;
pub mod handler;
pub mod response;
pub mod router;
pub mod utils;

use std::io::{Result as IOResult};
use std::net::SocketAddr;
use std::pin::{pin};
use std::sync::Arc;
use std::thread::{scope};

use rustls::{
    ServerConfig,
    pki_types::{
        pem::{PemObject},
        CertificateDer,
        PrivateKeyDer
    }
};
use tokio::net::{TcpListener};
use tokio_rustls::{rustls, TlsAcceptor};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::handler::http1x;
use crate::handler::http2x::H2_PREFACE;
use crate::router::{new_router, Router};

pub struct HTTP {
    acceptor: Option<TlsAcceptor>,
    listener: TcpListener,
    request_max_size: i64,
    router: Router,
}

pub async fn server(host: String, port: i32) -> IOResult<HTTP> {
    return Ok( HTTP {
        listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        request_max_size: 1024,
        acceptor: None,
        router: new_router(),
    });
}

fn get_tls_config(key: String, certs: String) -> IOResult<ServerConfig> {
    let certs = CertificateDer::pem_file_iter(certs)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let key = PrivateKeyDer::from_pem_file(key)
        .unwrap();
    let config: rustls::ServerConfig = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key).unwrap();
    return Ok(config);
}

pub async fn server_tls(host: String, port: i32, key: String, certs: String) -> IOResult<HTTP> {
    return Ok(HTTP {
        acceptor: Some(TlsAcceptor::from(Arc::new(get_tls_config(key, certs)?))),
        listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        request_max_size: 1024,
        router: new_router(),
    });
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

    async fn handle_stream<RW>(&mut self, stream: RW, addr:  SocketAddr) -> IOResult<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin
    {
        let mut reader = pin!(BufReader::new(stream));
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
                                // TODO: temp - find a better way... (failed borrowed twice...) - try function ->
                                tokio_scoped::scope(|scope| {
                                    // TODO: check point...
                                    scope.spawn(async {
                                        match acceptor.accept(stream).await {
                                            Ok(a) => {
                                                // match self.handle_stream(a, addr).await {
                                                //     Ok(_) => todo!(),
                                                //     Err(_) => todo!(),
                                                // }
                                            },
                                            Err(_) => todo!(),
                                        }
                                    });
                                });
                            },
                            None => {
                                // TODO: temp - find a better way...
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