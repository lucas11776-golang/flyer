use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result};
use bytes::Bytes;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, Incoming};
use rustls::ServerConfig;
use tokio::task::JoinSet;

use crate::server::protocol::udp::http3::Http3;
use crate::server::protocol::{ServerHandler, UdpHandler};
use crate::server::{GracefulShutdown, Server};
use crate::utils::mem::Instance;

pub mod http3;

pub struct Udp;

static ALPN_PROTOCOLS: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| {
    vec![
        b"h3".to_vec(),
        b"h3-34".to_vec(),
        b"h3-33".to_vec(),
        b"h3-32".to_vec(),
        b"h3-31".to_vec(),
        b"h3-30".to_vec(),
        b"h3-29".to_vec(),
        b"h3-28".to_vec(),
        b"h3-27".to_vec(),
        b"h3-26".to_vec(),
        b"h3-25".to_vec(),
        b"h3-24".to_vec(),
        b"h3-23".to_vec(),
        b"h3-22".to_vec(),
        b"h3-21".to_vec(),
        b"h3-20".to_vec(),
        b"h3-19".to_vec(),
        b"h3-18".to_vec(),
        b"h3-17".to_vec(),
        b"h3-16".to_vec(),
        b"h3-15".to_vec(),
        b"h3-14".to_vec(),
        b"h3-13".to_vec(),
        b"h3-12".to_vec(),
        b"h3-11".to_vec(),
        b"h3-10".to_vec(),
        b"h3-09".to_vec(),
        b"h3-08".to_vec(),
        b"h3-07".to_vec(),
        b"h3-06".to_vec(),
        b"h3-05".to_vec(),
        b"h3-04".to_vec(),
        b"h3-03".to_vec(),
        b"h3-02".to_vec(),
        b"h3-01".to_vec(),
        b"h3-00".to_vec(),
    ]
});

impl ServerHandler for Udp {
    async fn listen(instance: Instance<Server>) {
        let (addr_str, server_config) = {
            let server = instance.as_ref();
            let config = match &server.server_config {
                Some(config) => config.clone(),
                None => return,
            };
            (server.address().to_string(), config)
        };

        let endpoint: Endpoint = match Self::get_endpoint(&addr_str, server_config) {
            Ok(ep) => ep,
            Err(_) => {
                /* TODO need to change panic does not catch all application errors */
                return;
            }
        };

        match &instance.as_ref().graceful_shutdown {
            Some(callback) => Self::accept_with_graceful_shutdown(instance, endpoint, callback).await,
            None => Self::accept(instance, endpoint).await,
        }
    }
}

impl Udp {
    async fn accept(instance: Instance<Server>, endpoint: Endpoint) {
        while let Some(incoming) = endpoint.accept().await {
            tokio::spawn(async move {
                if let Err(_) = Self::on_incoming(instance, incoming).await {
                    /* TODO need to change panic does not catch all application errors */
                }
            });
        }
    }

    async fn accept_with_graceful_shutdown(
        instance: Instance<Server>,
        endpoint: Endpoint,
        graceful_shutdown: &GracefulShutdown,
    ) {
        let mut tasks = JoinSet::new();

        loop {
            tokio::select! {
                result = endpoint.accept() => {
                    match result {
                        Some(incoming) => {
                            tasks.spawn(async move {
                                if let Err(_) = Self::on_incoming(instance, incoming).await {
                                    /* TODO need to change panic does not catch all application errors */
                                }
                            });
                        },
                        None => {  },
                    }
                }

                _ = graceful_shutdown() => {
                    break;
                }
            }
        }

        drop(endpoint);

        while let Some(result) = tasks.join_next().await {
            if let Err(_) = result {
                /* TODO need to change panic does not catch all application errors */
            }
        }
    }


    fn get_endpoint(address: &str, mut server_config: ServerConfig) -> Result<Endpoint> {
        server_config.alpn_protocols = ALPN_PROTOCOLS.clone();

        let quic_config = QuicServerConfig::try_from(server_config)
            .map_err(|e| anyhow::anyhow!("Invalid QUIC TLS config: {e}"))?;

        let quinn_config = quinn::ServerConfig::with_crypto(Arc::new(quic_config));

        let socket_addr: SocketAddr = address
            .parse()
            .with_context(|| format!("Invalid socket address: {address}"))?;

        Endpoint::server(quinn_config, socket_addr)
            .with_context(|| format!("Failed to bind UDP endpoint on {socket_addr}"))
    }

    async fn on_incoming(server: Instance<Server>, incoming: Incoming) -> Result<()> {
        let conn = incoming.await?;
        let addr = conn.remote_address();

        match h3::server::Connection::<h3_quinn::Connection, Bytes>::new(h3_quinn::Connection::new(conn)).await {
            Ok(connection) => Http3::new(server.clone(), addr).handle(connection).await,
            Err(err) => Err(err.into()),
        }
    }
}