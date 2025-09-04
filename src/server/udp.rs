use std::pin::Pin;
use std::sync::Arc;
use std::io::Result as IOResult;

use bytes::Bytes;
use h3::server::RequestResolver;
use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};
use quinn::{Endpoint, Incoming, ServerConfig};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};

use crate::request::{Files, Request};
use crate::server::{get_server_config, HTTP3};
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
            let http = &self.http; // Borrow self.http

            tokio_scoped::scope(|scope| {
                scope.spawn(async move {
                    match incoming.await {
                        Ok(conn) => {
                            let mut connection: h3::server::Connection<h3_quinn::Connection, Bytes> = h3::server::Connection::new(h3_quinn::Connection::new(conn))
                                .await
                                .unwrap();

                            while let Ok(Some(resolver)) = connection.accept().await {
                                tokio_scoped::scope(|scope| {
                                    scope.spawn(async move {
                                        let (req, mut stream) = resolver.resolve_request().await.unwrap();
                                        let mut headers = Values::new();

                                        for (k, v) in req.headers() {
                                            headers.insert(k.to_string(), v.to_str().unwrap().to_string());
                                        }

                                        let request= Request{
                                            ip: "127.0.0.1".to_owned(),
                                            host: "".to_string(),
                                            method: req.method().to_string(),
                                            path: req.uri().path().to_string(),
                                            parameters: Values::new(),
                                            protocol: HTTP3.to_string(),
                                            headers: headers,
                                            body: vec![],
                                            values: Values::new(),
                                            files: Files::new(),
                                        };

                                        println!("request: {:?} - address: {:?}", request, http.address());
                                    });
                                });
                            }
                        }
                        Err(err) => {
                            eprintln!("Connection error: {}", err);
                        }
                    }
                });
            });
        }

        self.listener.wait_idle().await;
    }
}









































// impl <'a>UdpServer<'a> {
//     pub async fn new(http: &'a mut HTTP) -> UdpServer<'a> {
//         return UdpServer{
//             listener: Endpoint::server(UdpServer::get_config(http).unwrap(), http.address().parse().unwrap()).unwrap(),
//             http: http,
//         }
//     }

//     fn get_config(http: &'a mut HTTP) -> IOResult<ServerConfig> {
//         let mut config = get_server_config(&http.tls.as_ref().unwrap()).unwrap();

//         config.alpn_protocols = vec![
//             b"h3".to_vec(),
//             b"h3-29".to_vec(),
//             b"h3-32".to_vec(),
//             b"h3-34".to_vec(),
//         ];

//         return Ok(ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap())));
//     }

//     async fn new_connection(&mut self, http: Pin<Box<&mut HTTP>>, conn: Connection) -> IOResult<()> {
//         let mut connection: h3::server::Connection<h3_quinn::Connection, Bytes> = h3::server::Connection::new(h3_quinn::Connection::new(conn))
//             .await
//             .unwrap();

//         while let Ok(Some(resolver)) = connection.accept().await {
//             tokio_scoped::scope(|scope| {
//                 scope.spawn(async {

//                 })
//             });


//             // let http = Pin::new(Box::new(self.http));



//                 // tokio::spawn(UdpServer::handle_request(&mut http, resolver));

//             // tokio::spawn(async move {
//             // });
//         }

//         // TODO: find out why type will not live long enough.
   

//         Ok(())
//     }

//     pub async fn listen(& mut self) {
//         while let Some(incoming) = self.listener.accept().await {

          



//                 // UdpServer::new_connection(self.http, conn)


//             tokio_scoped::scope(|scope| {
//                 scope.spawn(async {
//                     match incoming.await {
//                         Ok(conn) => self.new_connection(conn).await.unwrap(),
//                         Err(err) => {} // TODO: Log
//                     }
//                 });
//             });
//         }
//     }

//     async fn handle_request<C>(http: &mut HTTP, resolver: RequestResolver<C, Bytes>)
//     where
//         C: h3::quic::Connection<Bytes>
//     {
//         let (req, mut stream) = resolver.resolve_request().await.unwrap();
//         println!("Received request: {} {}", req.method(), req.uri());

//         let body = "<h1>Hello World</h1>";

//         let response = http::Response::builder()
//             .status(200)
//             .header("Content-Type", "text/html")
//             .header("Content-Length", format!("{}", body.len()))
//             .body(()) // <-- must be ()
//             .unwrap();


//         match stream.send_response(response).await {
//             Ok(_) => {
//                 match stream.send_data(Bytes::from(body)).await {
//                     Ok(_) => todo!(),
//                     Err(err) => {
//                         // TODO: Error handle
//                     },
//                 }
//             }
//             Err(err) => {
//                 // TODO: Error handle
//             },
//         }

//         match stream.finish().await {
//             Ok(_) => {},
//             Err(_) => {
//                 // TODO: Error handle
//             },
//         }
//     }

// }