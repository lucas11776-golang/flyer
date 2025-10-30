use std::sync::Arc;
use std::io::Result;

use bytes::Bytes;
use h3_quinn::quinn::crypto::rustls::QuicServerConfig;
use quinn::{
    Connection as QuinnConnection,
    Endpoint,
    ServerConfig
};
use h3::server::Connection as H3ServerConnection;
use h3_quinn::Connection as H3Connection;

use crate::response::Response;
use crate::server::handler::http3;
use crate::server::{get_server_config};
use crate::HTTP;

pub(crate) struct UdpServer<'a> {
    http: &'a mut HTTP,
    listener: Endpoint,
}

impl <'a>UdpServer<'a> {    
    pub async fn new(http: &'a mut HTTP) -> Result<UdpServer<'a>> {
        return Ok(Self {
            listener: UdpServer::get_endpoint(http).unwrap(),
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


                    // let (stream, req) = handler.handle().await.unwrap();
                    // let res = self.http.router.match_web_routes(req, Response::new()).await.unwrap();

                    // handler.write(stream,&mut self.http.render_response_view(res)).await.unwrap();

                });
            });
        }
    }

    async fn get_h3_server_connection(&mut self, conn: QuinnConnection) -> H3ServerConnection<H3Connection, Bytes> {
        return H3ServerConnection::new(H3Connection::new(conn))
            .await
            .unwrap();
    }

    fn get_endpoint(http: &'a mut HTTP) -> Result<Endpoint> {
        Ok(Endpoint::server(UdpServer::get_config(http).unwrap(), http.address().parse().unwrap()).unwrap())
    }

    fn get_config(http: &'a mut HTTP) -> Result<ServerConfig> {
        let mut config = get_server_config(&http.tls.as_ref().unwrap())?;

        config.alpn_protocols = UdpServer::get_alpn_protocols();

        Ok(ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap())))
    }

    fn get_alpn_protocols() -> Vec<Vec<u8>> {
        return vec![
            b"h3".to_vec(),
            b"h3-29".to_vec(),
            b"h3-32".to_vec(),
            b"h3-34".to_vec(),
        ];
    }
}