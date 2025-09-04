use std::pin::Pin;
use std::sync::Arc;
use std::io::Result as IOResult;

use bytes::Bytes;
use h3::server::{
    RequestResolver,
    RequestStream
};
use h3_quinn::{
    BidiStream,
    quinn::crypto::rustls::QuicServerConfig
};
use quinn::{
    Connection,
    Endpoint,
    ServerConfig
};

use crate::request::{Files, Request};
use crate::server::{
    get_server_config,
    HTTP3
};
use crate::utils::url::parse_query_params;
use crate::utils::Values;
use crate::HTTP;

pub struct UdpServer<'a> {
    listener: Pin<Box<Endpoint>>,
    http: Pin<Box<&'a mut HTTP>>,
}

impl<'a> UdpServer<'a> {
    pub async fn new(http: &'a mut HTTP) -> UdpServer<'a> {
        return UdpServer {
            listener: Box::pin(Endpoint::server(UdpServer::get_config(http).unwrap(), http.address().parse().unwrap()).unwrap()),
            http: Box::pin(http),
        }
    }

    fn get_config(http: &'a mut HTTP) -> IOResult<ServerConfig> {
        let mut config = get_server_config(&http.tls.as_ref().unwrap())?;

        config.alpn_protocols = vec![
            b"h3".to_vec(),
            b"h3-29".to_vec(),
            b"h3-32".to_vec(),
            b"h3-34".to_vec(),
        ];

        Ok(ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap())))
    }

    pub async fn listen(&mut self) {
        while let Some(incoming) = self.listener.accept().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    match incoming.await {
                        Ok(conn) => self.new_connection(conn).await,
                        Err(_) => {} // TODO: Log
                    }
                });
            });
        }

        self.listener.wait_idle().await;
    }
    
    async fn new_connection(&mut self, conn: Connection) {
        let mut connection: h3::server::Connection<h3_quinn::Connection, Bytes> = h3::server::Connection::new(h3_quinn::Connection::new(conn))
            .await
            .unwrap();

        while let Ok(Some(resolver)) = connection.accept().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(self.handle_resolver(resolver));
            });
        }
    }

    async fn handle_resolver(&mut self, resolver: RequestResolver<h3_quinn::Connection, Bytes>) {
        let (req, mut stream) = resolver.resolve_request().await.unwrap();
        let mut headers = Values::new();

        for (k, v) in req.headers() {
            headers.insert(k.to_string(), v.to_str().unwrap().to_string());
        }

        let host = headers
            .get("host")
            .cloned()
            .or_else(|| headers.get(":authority").cloned())
            .unwrap_or_default();

        let mut request= Request{
            ip: "127.0.0.1".to_owned(),
            host: host.to_string(),
            method: req.method().to_string(),
            path: req.uri().path().to_string(),
            query: parse_query_params(req.uri().query().unwrap_or("")),
            protocol: HTTP3.to_string(),
            headers: headers,
            body: vec![],
            values: Values::new(),
            files: Files::new(),
        };

        self.handle_request(&mut request, &mut stream).await;
    }

    async fn handle_request<'s>(&mut self, req: &'s mut Request, stream: &'s RequestStream<BidiStream<Bytes>, Bytes>) {


        


        println!("REQUEST: {:?}", req);
    }
}