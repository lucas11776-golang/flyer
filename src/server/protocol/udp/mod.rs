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
use crate::server::Server;
use crate::utils::mem::Instance;
use crate::utils::shutdown::shutdown_signal;

pub mod http3;

pub struct Udp;

static ALPN_PROTOCOLS: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| {
    vec![
        b"h3".to_vec(),
        b"h3-29".to_vec(),
        b"h3-32".to_vec(),
        b"h3-34".to_vec(),
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

        let endpoint = match Self::get_endpoint(&addr_str, server_config) {
            Ok(ep) => ep,
            Err(_) => {
                /* TODO need to change panic does not catch all application errors */
                return;
            }
        };

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

                _ = shutdown_signal() => {
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
}

impl Udp {
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