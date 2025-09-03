use std::{io::Result, sync::Arc};

use crate::{server::get_server_config, HTTP};
use quinn::{Endpoint, ServerConfig};
use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};

use bytes::Bytes;
use h3::server::RequestResolver;
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::io::Result as IOResult;

pub struct UdpServer<'a> {
    listener: Endpoint,
    http: &'a mut HTTP
}

impl <'a>UdpServer<'a> {
    pub async fn new(http: &'a mut HTTP) -> UdpServer<'a> {
        return UdpServer{
            listener: Endpoint::server(UdpServer::get_config(http).unwrap(), http.address().parse().unwrap()).unwrap(),
            http: http,
        }
    }

    fn get_config(http: &'a mut HTTP) -> Result<ServerConfig> {
        let mut config = get_server_config(&http.tls.as_ref().unwrap()).unwrap();

        config.alpn_protocols = vec![
            b"h3".to_vec(),
            b"h3-29".to_vec(),
            b"h3-32".to_vec(),
            b"h3-34".to_vec(),
        ];

        return Ok(ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap())));
    }

    pub async fn listen(&mut self) {
        while let Some(new_conn) = self.listener.accept().await {
            tokio::spawn(async move {
                match new_conn.await {
                    Ok(conn) => {


                        println!("NEW HTTP3 Connection: {:?}", conn.remote_address().ip().to_string());

                        // let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                        //     .await
                        //     .unwrap();

                        // while let Ok(Some(resolver)) = h3_conn.accept().await {
                        //     // tokio::spawn(self.handle_request(resolver));
                        // }
                    }
                    Err(err) => {
                        eprintln!("Connection error: {}", err);
                    }
                }
            });
        }
    }

    async fn handle_request<C>(&mut self, resolver: RequestResolver<C, Bytes>)
    where
        C: h3::quic::Connection<Bytes>,
    {
        let (req, mut stream) = resolver.resolve_request().await.unwrap();
        println!("Received request: {} {}", req.method(), req.uri());

        let body = "<h1>Hello World</h1>";

        let response = http::Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .header("Content-Length", format!("{}", body.len()))
            .body(()) // <-- must be ()
            .unwrap();


        match stream.send_response(response).await {
            Ok(_) => {
                match stream.send_data(Bytes::from(body)).await {
                    Ok(_) => todo!(),
                    Err(err) => {
                        // TODO: Error handle
                    },
                }
            }
            Err(err) => {
                // TODO: Error handle
            },
        }

        match stream.finish().await {
            Ok(_) => {},
            Err(_) => {
                // TODO: Error handle
            },
        }
    }

}