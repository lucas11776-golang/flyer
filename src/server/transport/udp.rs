use crate::server::Server;

pub(crate) async fn listen(server: usize) {

}

// #[allow(static_mut_refs)]
// fn get_endpoint(mut config: ServerConfig) -> Result<Endpoint> {
//     config.alpn_protocols = ALPN_PROTOCOLS.to_vec();
//     let quinn_config = Arc::new(QuicServerConfig::try_from(config).unwrap());
//     let server_config = QuinnServerConfig::with_crypto(quinn_config);
//     Ok(Endpoint::server(server_config, unsafe { APPLICATION.address().parse().unwrap() }).unwrap())
// }

// async fn listener(listener: Endpoint) {
//     while let Some(incoming) = listener.accept().await {
//         tokio::spawn(async move {
//             let conn = incoming.await;

//             if conn.is_err() {
//                 return; // TODO: log
//             }
            
//             let server: StdResult<H3Server<H3Conn, Bytes>, ConnectionError> = H3Server::new(H3Conn::new(conn.unwrap())).await;

//             if server.is_err() {
//                 return; // TODO: log
//             }

//             listen_peer(server.unwrap()).await;
//         });
//     }
// }