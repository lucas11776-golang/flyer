use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};
use std::io::Result;

use bytes::Bytes;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, ServerConfig as QuinnServerConfig};
use h3_quinn::Connection as H3QuinnConn;

use h3::server::Connection as H3ServerConn;
use rustls::ServerConfig;

use crate::response::Response;
use crate::server::Server;
use crate::server::transport::handler::http3;

const ALPN_PROTOCOLS: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| vec![
    b"h3".to_vec(),
    b"h3-29".to_vec(),
    b"h3-32".to_vec(),
    b"h3-34".to_vec(),
]);

pub(crate) async fn listen(ptr: usize) {
    let server = Server::instance(ptr);
    
    match &server.server_config {
        Some(config) => listener(ptr, get_endpoint(server.address(), config.clone()).unwrap()).await,
        None => {},
    }
}

fn get_endpoint(address: String, mut config: ServerConfig) -> Result<Endpoint> {
    config.alpn_protocols = ALPN_PROTOCOLS.to_vec();
    let quinn_config = Arc::new(QuicServerConfig::try_from(config).unwrap());
    let server_config = QuinnServerConfig::with_crypto(quinn_config);
    return Ok(Endpoint::server(server_config, address.parse().unwrap()).unwrap());
}

async fn listener(ptr: usize, listener: Endpoint) {
    while let Some(incoming) = listener.accept().await {
        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => {
                    let addr = conn.remote_address();

                    match H3ServerConn::<H3QuinnConn, Bytes>::new(H3QuinnConn::new(conn)).await {
                        Ok(server) => listen_peer_server_connection(ptr, server, addr).await,
                        Err(_) => { /* Log */ },
                    }
                },
                Err(_) => { /* Log */ },
            };
        });
    }
}

async fn listen_peer_server_connection(ptr: usize, mut server: H3ServerConn<H3QuinnConn, Bytes>, addr: SocketAddr) {
    while let Ok(Some(incoming)) = server.accept().await {
        tokio::spawn(async move {
            let (request, stream) = incoming.resolve_request().await.unwrap();
            let mut handler = http3::Handler::new(request, stream, addr);
            let (mut req, mut res) = (handler.handle().await.unwrap(), Response::new());

            Server::instance(ptr).on_web_request(&mut req, &mut res).await.unwrap();
            handler.write(&mut req, &mut res).await.unwrap();
        });
    }
}