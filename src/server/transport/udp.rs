use std::sync::{Arc, LazyLock};
use std::io::Result;

use bytes::Bytes;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, ServerConfig as QuinnServerConfig};
use h3::server::Connection as H3Server;
use h3_quinn::Connection as H3Conn;
use rustls::ServerConfig;

use crate::GLOBAL_SERVER;
use crate::response::Response;
use crate::server::Server;
use crate::server::transport::handler::http3;

const ALPN_PROTOCOLS: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| vec![
    b"h3".to_vec(),
    b"h3-29".to_vec(),
    b"h3-32".to_vec(),
    b"h3-34".to_vec(),
]);

pub(crate) async fn listen(server_ptr: usize) {
    unsafe {
        let server = &*(server_ptr as *const &mut Server);

        match &server.server_config {
            Some(config) => listener(get_endpoint(server.address(), config.clone()).unwrap()).await,
            None => {},
        }
    }
}

fn get_endpoint(address: String, mut config: ServerConfig) -> Result<Endpoint> {
    config.alpn_protocols = ALPN_PROTOCOLS.to_vec();
    let quinn_config = Arc::new(QuicServerConfig::try_from(config).unwrap());
    let server_config = QuinnServerConfig::with_crypto(quinn_config);
    return Ok(Endpoint::server(server_config, address.parse().unwrap()).unwrap());
}

async fn listener(listener: Endpoint) {
    while let Some(incoming) = listener.accept().await {
        tokio::spawn(async move {
            match incoming.await {
                Ok(conn) => {
                    match H3Server::<H3Conn, Bytes>::new(H3Conn::new(conn)).await {
                        Ok(server) => listen_peer_server_connection(server).await,
                        Err(_) => { /* Log */ },
                    }
                },
                Err(_) => { /* Log */ },
            };
        });
    }
}

#[allow(static_mut_refs)]
async fn listen_peer_server_connection(mut server: H3Server<H3Conn, Bytes>) {
    while let Ok(Some(resolver)) = server.accept().await {
        tokio::spawn(async move {
            unsafe {
                let (request, stream) = resolver.resolve_request().await.unwrap();
                let mut handler = http3::Handler::new(request, stream);
                let (mut req, mut res) = (handler.handle().await.unwrap(), Response::new());

                GLOBAL_SERVER.get_mut().unwrap().on_web_request(&mut req, &mut res).await.unwrap(); // TODO: need to remove and use `server_ptr`

                handler.write(&mut req, &mut res).await.unwrap();
            }
        });
    }
}