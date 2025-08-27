pub mod request;
pub mod handler;
pub mod response;
pub mod router;
pub mod utils;
pub mod session;

pub type Values = HashMap<String, String>;

use std::collections::HashMap;
use std::io::{Result as IOResult};
use std::net::SocketAddr;
use std::pin::{pin};
use std::sync::Arc;
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

use crate::handler::{http1, http2};
use crate::handler::http2::{H2_PREFACE};
use crate::router::{new_group_router, GroupRouter, Router};
use crate::session::SessionManager;

pub struct HTTP {
    acceptor: Option<TlsAcceptor>,
    listener: TcpListener,
    request_max_size: i64,
    router: GroupRouter,
    pub(crate) session_manger: Option<SessionManager>,
}

pub async fn server<'a>(host: &str, port: i32) -> IOResult<HTTP> {
    return Ok(HTTP {
        listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        request_max_size: 1024,
        acceptor: None,
        router: new_group_router(),
        session_manger: None
    });
}

fn get_tls_config(key: &str, certs: &str) -> IOResult<ServerConfig> {
    // Retrieve the default cryptographic provider from the rustls library, which is based on the ring library.
    rustls::crypto::ring::default_provider()
        // Install the default cryptographic provider for use in rustls.
        .install_default()
        .unwrap();
    let certs = CertificateDer::pem_file_iter(certs)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let key = PrivateKeyDer::from_pem_file(key)
        .unwrap();
    let config: rustls::ServerConfig = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    return Ok(config);
}

pub async fn server_tls<'a>(host: &str, port: i32, key: &str, certs: &str) -> IOResult<HTTP> {
    return Ok(HTTP {
        acceptor: Some(TlsAcceptor::from(Arc::new(get_tls_config(key, certs)?))),
        listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        request_max_size: 1024,
        router: new_group_router(),
        session_manger: None,
    });
}

impl <'a>HTTP {
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

    pub fn session(&mut self, token: &str) -> &mut HTTP {
        // TODO: continue
        return self;
    }

    async fn handle_stream<RW>(&mut self, stream: RW, addr:  SocketAddr) -> IOResult<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        let mut reader = pin!(BufReader::new(stream));
        let buf = reader.fill_buf().await?;

        match buf.len() >= H2_PREFACE.len() && &buf[..H2_PREFACE.len()] == H2_PREFACE {
            true => http2::Handler::handle(self,reader, addr).await?,
            false => http1::Handler::handle(self,reader, addr).await?,
        }

        Ok(())
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

    async fn new_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send {
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

    pub async fn listen(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tokio_scoped::scope(|scope| {
                        scope.spawn(self.new_connection(stream, addr));
                    });
                },
                Err(err) => println!("{}", err),
            }
        }
    }

    pub fn router(&'a mut self) -> Router<'a> {
        return Router{
            router: &mut self.router,
            path: vec!["/".to_string()],
            middleware: vec![],
        };
    }
}