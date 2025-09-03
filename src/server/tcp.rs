use std::{
    io::Result,
    net::SocketAddr,
    sync::Arc
};
use std::pin::{pin};

use tokio::net::{TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::handler::http1;
use crate::handler::http2::{H2_PREFACE};
use crate::server::{get_server_config, RoutesCallback, WebCallback};

pub struct TcpServer<'a> {
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    callback: Option<RoutesCallback<'a>>,
    web_callback: Option<WebCallback<'a>>,
}

#[derive(Clone)]
pub struct Tls {
    pub key_path: String,
    pub cert_path: String
}

// TODO: bad design - can not pass self.callback in request...
impl <'a>TcpServer<'a> {
    pub async fn new(host: &str, port: i32, tls: Option<Tls>) -> TcpServer {
        match tls {
            Some(tls) => TcpServer {
                listener: TcpListener::bind(format!("{0}:{1}", host, port)).await.unwrap(),
                acceptor: Some(TlsAcceptor::from(Arc::new(get_server_config(tls.key_path.as_str(), tls.cert_path.as_str()).unwrap()))),
                callback: None,
                web_callback: None,
            },
            None => TcpServer {
                listener: TcpListener::bind(format!("{0}:{1}", host, port)).await.unwrap(),
                acceptor: None,
                callback: None,
                web_callback: None,
            },
        }
    }

    pub fn on_request(&mut self, callback: WebCallback<'a>) -> &mut Self {
        // self.callback = Some(callbacks);

        self.web_callback = Some(callback);

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
            false => {

                // TODO: temp refactor

                // match &self.callback {
                //     Some(callbacks) => {
                //         http1::Handler::handle(callbacks.web, rw, addr).await?
                //     },
                //     None => {},
                // }


                // 
            },
        }

        Ok(())
    }

}


