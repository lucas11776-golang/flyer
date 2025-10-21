use std::{collections::HashMap, io::Result};
use std::net::SocketAddr;
use std::pin::Pin;

use bytes::Bytes;

use h2::{server, server::{SendResponse}};
use http::{HeaderMap, Request as HttpRequest, Response as HttpResponse};
use reqwest::Url;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::response::{Response};
// use crate::{response::new_response, server::handler::RequestHandler, utils::Values, HTTP};
use crate::request::{Headers, Request};
use crate::utils::url::parse_query_params;
use crate::utils::Values;
use crate::HTTP;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub struct Handler { }


// impl <'a> Handler {
//     pub fn new() -> Self {
//         return Self {}
//     }

//     pub async fn handle<RW>(&mut self, http: &'a mut HTTP, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
//     where
//         RW: AsyncRead + AsyncWrite + Unpin + std::marker::Send
//     {
//         // let mut conn = server::handshake(&mut rw).await.unwrap();

//         // while let Some(result) = conn.accept().await {
//         //     let (req, response) = result.unwrap();

//         //     tokio_scoped::scope(|scope| {
//         //         scope.spawn(async {
//         //             let _ = self.new_request( http,req, response, addr).await;
//         //         });
//         //     });
//         // }

//         return Ok(());
//     }

//     async fn connect<RW>(&mut self, http: &'a mut HTTP, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr)
//     where
//         RW: AsyncRead + AsyncWrite + Unpin + Send
//     {
//         let mut conn = server::handshake(&mut rw).await.unwrap();

//         while let Some(result) = conn.accept().await {
//             let (req, response) = result.unwrap();

//             // tokio_scoped::scope(|scope| {
//             //     scope.spawn(async {
//             //         let _ = self.new_request( http,req, response, addr).await;
//             //     });
//             // });
//         }
//     }

//     async fn new_request(&mut self, http: &'a mut HTTP, request: HttpRequest<h2::RecvStream>, send: SendResponse<Bytes>, addr: SocketAddr) -> std::io::Result<()> {
//         let method = request.method().to_string();
//         let path = Url::parse(request.uri().to_string().as_str()).unwrap().path().to_string();
//         let query = parse_query_params(request.uri().query().unwrap_or(""))?;
//         let mut body = Vec::new();
//         let headers = self.hashmap_to_headers(request.headers());
//         let mut recv = request.into_body();

//         while let Some(chunk) = recv.data().await.transpose().unwrap() {
//             body.extend_from_slice(&chunk);
//         }

//         let host = headers
//             .get("host")
//             .cloned()
//             .or_else(|| headers.get(":authority").cloned())
//             .unwrap_or_default();

//         let req = Request {
//             ip: addr.ip().to_string(),
//             host: host,
//             method: method,
//             path: path,
//             parameters: Values::new(),
//             query: query,
//             protocol: "HTTP/2.0".to_string(),
//             headers: headers,
//             body: body,
//             values: HashMap::new(),
//             files: HashMap::new(),
//         };

//         self.handle_request(http, req, send).await?;

//         Ok(())
//     }

//     fn hashmap_to_headers(&mut self, map: &HeaderMap) -> Headers {
//         let mut headers = Headers::new();

//         for (k, v) in map.iter() {
//             headers.insert(
//                 k.as_str().to_string(),
//                 v.to_str().unwrap_or_default().to_string()
//             );
//         }

//         return headers;
//     }

//     async fn handle_request(&mut self, http: &'a mut HTTP, mut req: Request, mut send:  SendResponse<Bytes>) -> Result<()> {
//         // let mut response = new_response();
//         // let response = RequestHandler::web(http, &mut req, &mut response).await?;

//         // let mut builder = HttpResponse::builder().status(response.status_code);

//         // for (k, v) in &mut response.headers {
//         //     builder = builder.header(k.clone(), v.clone());
//         // }

//         // return Ok(
//         //     send.send_response(builder.body(()).unwrap(), false)
//         //         .unwrap()
//         //         .send_data(Bytes::from(response.body.clone()), true)
//         //         .unwrap()
//         // )


//         Ok(())
//     }
// }


use futures::future::{BoxFuture, Future, FutureExt};



