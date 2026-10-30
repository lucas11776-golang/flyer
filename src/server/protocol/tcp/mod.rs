use std::io::{Error, ErrorKind::Unsupported};
use std::net::SocketAddr;

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader},
    net::TcpListener,
    task::JoinSet
};
use tokio_rustls::TlsAcceptor;

use crate::server::protocol::TcpHandler;
use crate::server::protocol::tcp::http2::{H2_PREFACE, Http2};
use crate::utils::shutdown::shutdown_signal;
use crate::{
    server::{
        Server,
        protocol::ServerHandler,
        protocol::{Protocol, tcp::http1::Http1},
    },
    utils::{mem::Instance, server::get_tls_acceptor},
};

pub mod http1;
pub mod http2;

pub struct Tcp;

impl ServerHandler for Tcp {
    async fn listen(server: Instance<Server>) {
        let listener = TcpListener::bind(server.as_ref().address())
            .await
            .expect("Failed to bind TCP listener");

        let tls = server
            .as_mut()
            .server_config
            .clone()
            .map(|cfg| get_tls_acceptor(cfg));

        let mut tasks = JoinSet::new();

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            let tls = tls.clone();
                            tasks.spawn(Self::process_stream(server.clone(), tls, stream, addr));
                        },
                        Err(_) => {
                            /* TODO need to change panic does not catch all application errors */
                        },
                    }
                }

                _ = shutdown_signal() => {
                    break;
                }
            }
        }

        drop(listener);

        while let Some(result) = tasks.join_next().await {
            if let Err(_) = result {
                /* TODO need to change panic does not catch all application errors */
            }
        }
    }
}

impl Tcp {
    async fn process_stream(
        server: Instance<Server>,
        tls: Option<TlsAcceptor>,
        stream: tokio::net::TcpStream,
        addr: SocketAddr,
    ) {
        let result = match tls {
            Some(acceptor) => match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    Self::handle_connection(server, addr, BufReader::new(tls_stream)).await
                }
                Err(err) => Err(err.into()),
            },
            None => Self::handle_connection(server, addr, BufReader::new(stream)).await,
        };

        // TODO: handle error
        if let Err(_) = result {
            // TODO: log error (e.g., tracing::error!)
        }
    }

    async fn handle_connection<RW>(
        server: Instance<Server>,
        addr: SocketAddr,
        mut rw: BufReader<RW>,
    ) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
    {
        match Self::determine_protocol(&mut rw).await? {
            Protocol::HTTP1 => Http1::new(server, addr).handle(rw).await,
            Protocol::HTTP2 => Http2::new(server, addr).handle(rw).await,
            Protocol::HTTP3 => Err(Error::new(Unsupported, "HTTP/3 is only supported by UDP protocol").into())
        }
    }

    async fn determine_protocol<RW>(rw: &mut BufReader<RW>) -> Result<Protocol>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
    {
        let buf = rw.fill_buf().await?;

        if buf.len() >= H2_PREFACE.len() && &buf[..H2_PREFACE.len()] == H2_PREFACE  {
            return Ok(Protocol::HTTP2)
        }

        Ok(Protocol::HTTP1)
    }
}
