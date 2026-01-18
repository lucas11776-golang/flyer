use std::sync::{Arc, LazyLock};
use std::io::Result;
use std::result::Result as StdResult;

use bytes::Bytes;
use h3::error::ConnectionError;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{Endpoint, ServerConfig as QuinnServerConfig};
use h3::server::Connection as H3Server;
use h3_quinn::Connection as H3Conn;
use rustls::ServerConfig;

use crate::request::Request;
use crate::response::Response;
use crate::server::handler::http3;
use crate::server::helpers::{Handler, RequestHandler};
use crate::server::protocol::http::APPLICATION;

const ALPN_PROTOCOLS: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| vec![
    b"h3".to_vec(),
    b"h3-29".to_vec(),
    b"h3-32".to_vec(),
    b"h3-34".to_vec(),
]);

#[allow(static_mut_refs)]
pub(crate) async fn listen() {
    unsafe {
        if let Some(config) = &APPLICATION.server_config {
            listener(get_endpoint(config.clone()).unwrap()).await;
        }
    }
}

#[allow(static_mut_refs)]
fn get_endpoint(mut config: ServerConfig) -> Result<Endpoint> {
    config.alpn_protocols = ALPN_PROTOCOLS.to_vec();
    let quinn_config = Arc::new(QuicServerConfig::try_from(config).unwrap());
    let server_config = QuinnServerConfig::with_crypto(quinn_config);
    Ok(Endpoint::server(server_config, unsafe { APPLICATION.address().parse().unwrap() }).unwrap())
}

async fn listener(listener: Endpoint) {
    while let Some(incoming) = listener.accept().await {
        tokio::spawn(async move {
            let conn = incoming.await;

            if conn.is_err() {
                return; // TODO: log
            }
            
            let server: StdResult<H3Server<H3Conn, Bytes>, ConnectionError> = H3Server::new(H3Conn::new(conn.unwrap())).await;

            if server.is_err() {
                return; // TODO: log
            }

            listen_peer(server.unwrap()).await;
        });
    }
}

async fn listen_peer(mut server: H3Server<H3Conn, Bytes>) {
    while let Ok(Some(resolver)) = server.accept().await {
        tokio::spawn(async move {
            let (request, stream) = resolver.resolve_request().await.unwrap();
            let mut handler = http3::Handler::new(request, stream);
            let req = handler.handle().await.unwrap();
            let res = Response::new();
            let result_handle = handle(req, res).await;

            if result_handle.is_err() {
                return;
            }

            let (mut req, mut res) = result_handle.unwrap();

            if let Ok(_) = handler.write(&mut req, &mut res).await { }
        });
    }
}

#[allow(static_mut_refs)]
async fn handle<'h>(req: Request, res: Response) -> Result<(Request, Response)> {
    unsafe {
        let handler = RequestHandler::new();
        let (mut req, mut res) = handler.setup(req, res).await.unwrap();

        res.referer = req.header("referer");

        let resp = APPLICATION.router.web_match(&mut req, &mut res).await;

        if resp.is_none() && APPLICATION.assets.is_some() {
            (req, res) = APPLICATION.assets.as_mut().unwrap().handle(req, res).unwrap();
        }

        return Ok(handler.teardown(req, res).await.unwrap());
    }
}