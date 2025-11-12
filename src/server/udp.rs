use std::sync::Arc;
use std::io::Result;

use bytes::Bytes;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{
    Connection as QuinnConnection,
    Endpoint,
    ServerConfig as QuinnServerConfig
};
use h3::server::Connection as H3ServerConnection;
use h3_quinn::Connection as H3Connection;
use rustls::ServerConfig;

use crate::response::Response;
use crate::server::handler::http3;
use crate::HTTP;

pub(crate) struct UdpServer<'a> {
    http: &'a mut HTTP,
    listener: Endpoint,
}

impl <'a>UdpServer<'a> {    
    pub async fn new(http: &'a mut HTTP, tls: ServerConfig) -> Result<UdpServer<'a>> {
        return Ok(Self {
            listener: UdpServer::get_endpoint(http.address().clone(), tls).unwrap(),
            http: http,
        });
    }

    pub async fn listen(&mut self) {
        while let Some(incoming) = self.listener.accept().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    match incoming.await {
                        Ok(conn) => self.connection(conn).await,
                        Err(_) => {} // TODO: Log
                    }
                });
            });
        }
    }

    async fn connection(&mut self, conn: QuinnConnection) {
        let mut server = self.get_h3_server_connection(conn).await;

        while let Ok(Some(resolver)) = server.accept().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let (request, stream) = resolver.resolve_request().await.unwrap();
                    let mut handler = http3::Handler::new(request, stream);
                    let mut req = handler.handle().await.unwrap();
                    let mut res = Response::new();
                    
                    self.http.router.match_web_routes(&mut req, &mut res).await.unwrap();

                    if res.view.is_some() && self.http.view.is_some() {
                        res = self.http.view.as_mut().unwrap().render(res).unwrap();
                    }

                    handler.write(&mut res).await.unwrap();
                });
            });
        }
    }

    async fn get_h3_server_connection(&mut self, conn: QuinnConnection) -> H3ServerConnection<H3Connection, Bytes> {
        return H3ServerConnection::new(H3Connection::new(conn))
            .await
            .unwrap();
    }

    fn get_endpoint(address: String, mut config: ServerConfig) -> Result<Endpoint> {
        config.alpn_protocols = UdpServer::setup_alpn_protocols();
        let quinn_config = QuinnServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap()));

        Ok(Endpoint::server(quinn_config, address.parse().unwrap()).unwrap())
    }

    fn setup_alpn_protocols() -> Vec<Vec<u8>> {
        return vec![
            b"h3".to_vec(),
            b"h3-29".to_vec(),
            b"h3-32".to_vec(),
            b"h3-34".to_vec(),
        ];
    }
}