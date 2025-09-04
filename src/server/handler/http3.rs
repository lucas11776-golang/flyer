// use bytes::Bytes;
// use flyer::server::get_server_config;
// use h3::server::RequestResolver;
// use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};
// use quinn::{Endpoint, ServerConfig};
// use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
// use std::io::Result as IOResult;
// use std::sync::Arc;

// #[tokio::main]
// async fn main() -> IOResult<()> {
//     let mut config = get_server_config("host.key", "host.cert")?;

//     config.alpn_protocols = vec![
//         b"h3".to_vec(),
//         b"h3-29".to_vec(),
//         b"h3-32".to_vec(),
//         b"h3-34".to_vec(),
//     ];

//     let server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(config).unwrap()));
//     let endpoint = Endpoint::server(server_config, "127.0.0.1:9999".parse().unwrap())?;

//     println!("Server running on https://127.0.0.1:9999 (HTTP/3)");

//     while let Some(new_conn) = endpoint.accept().await {
//         tokio::spawn(async move {
//             match new_conn.await {
//                 Ok(conn) => {
//                     let mut connection: h3::server::Connection<h3_quinn::Connection, Bytes> = h3::server::Connection::new(h3_quinn::Connection::new(conn))
//                         .await
//                         .unwrap();

//                     while let Ok(Some(resolver)) = connection.accept().await {
//                         tokio::spawn(handle_request(resolver));
//                     }
//                 }
//                 Err(err) => {
//                     eprintln!("Connection error: {}", err);
//                 }
//             }
//         });
//     }

//     endpoint.wait_idle().await;

//     Ok(())
// }

// // fn get_tls_config(key: &str, certs: &str) -> IOResult<rustls::ServerConfig> {
// //     rustls::crypto::ring::default_provider()
// //         .install_default()
// //         .unwrap();

// //     let certs = CertificateDer::pem_file_iter(certs)
// //         .unwrap()
// //         .collect::<Result<Vec<_>, _>>()
// //         .unwrap();
// //     let key = PrivateKeyDer::from_pem_file(key).unwrap();

// //     let mut config: rustls::ServerConfig = rustls::ServerConfig::builder()
// //         .with_no_client_auth()
// //         .with_single_cert(certs, key)
// //         .unwrap();



// //     Ok(config)
// // }

// async fn handle_request<C>(resolver: RequestResolver<C, Bytes>)
// where
//     C: h3::quic::Connection<Bytes>,
// {
//     let (req, mut stream) = resolver.resolve_request().await.unwrap();
//     println!("Received request: {} {}", req.method(), req.uri());

//     let body = "<h1>Hello World</h1>";

//     let response = http::Response::builder()
//         .status(200)
//         .header("Content-Type", "text/html")
//         .header("Content-Length", format!("{}", body.len()))
//         .body(()) // <-- must be ()
//         .unwrap();


//     match stream.send_response(response).await {
//         Ok(_) => {
//             match stream.send_data(Bytes::from(body)).await {
//                 Ok(_) => todo!(),
//                 Err(err) => {
//                     // TODO: Error handle
//                 },
//             }
//         }
//         Err(err) => {
//             // TODO: Error handle
//         },
//     }

//     match stream.finish().await {
//         Ok(_) => {},
//         Err(_) => {
//             // TODO: Error handle
//         },
//     }
// }