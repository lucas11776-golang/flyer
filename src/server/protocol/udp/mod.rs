use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result};
use bytes::Bytes;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, Incoming};
use rustls::ServerConfig;

use crate::server::protocol::udp::http3::Http3;
use crate::server::protocol::{ServerHandler, UdpHandler};
use crate::server::Server;
use crate::utils::mem::Instance;

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
            Err(err) => {
                eprintln!("Failed to bind QUIC/HTTP3 endpoint on {addr_str}: {err}");
                return;
            }
        };

        while let Some(incoming) = endpoint.accept().await {
            let instance = instance.clone();

            tokio::spawn(async move {
                if let Err(err) = Self::on_incoming(instance, incoming).await {
                    eprintln!("HTTP/3 connection error: {err}");
                }
            });
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