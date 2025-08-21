use bytes::Bytes;
use h2::server;
use http::{HeaderMap, Request as H2Request, Response as H2Response, StatusCode};
use std::collections::HashMap;
use std::io::{Error as IoError, ErrorKind};
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{HTTP as Server};
use crate::request::{Headers, Request};
use crate::utils::url::parse_query_params;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub struct Handler<'a> {
    http: &'a mut Server,
    addr: SocketAddr,
}

impl <'a> Handler<'a> {
    pub async fn handle<RW>(server: &'a mut Server, rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> std::io::Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + std::marker::Send
    {
        let mut handler: Handler<'_> = Handler{
            http: server,
            addr: addr
        };

        handler.connect(rw).await;

        return Ok(());
    }

    async fn connect<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>)
    where
        RW: AsyncRead + AsyncWrite + Unpin + std::marker::Send
    {
        let mut conn = server::handshake(&mut rw).await.unwrap();

        while let Some(result) = conn.accept().await {
            let (req, response) = result.unwrap();

            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let _ = self.handle_h2_stream( req, response).await;
                });
            });
        }
    }

    async fn handle_h2_stream<>(&mut self , request: H2Request<h2::RecvStream>, mut response: server::SendResponse<Bytes>) -> std::io::Result<()> {
        let method = request.method().to_string();
        let path = request.uri().to_string();
        let parameters = parse_query_params(request.uri().query().unwrap_or(""));
        let mut body = Vec::new();
        let headers = self.hashmap_to_headers(request.headers());
        let mut recv = request.into_body();

        while let Some(chunk) = recv
            .data()
            .await
            .transpose()
            .map_err(|e| IoError::new(ErrorKind::Other, format!("h2 recv data error: {e}")))? 
        {
            body.extend_from_slice(&chunk);
        }

        let host = headers
            .get("host")
            .cloned()
            .or_else(|| headers.get(":authority").cloned())
            .unwrap_or_default();

        let req = Request {
            host: host,
            method: method,
            path: path,
            parameters: parameters,
            protocol: "HTTP/2.0".to_string(),
            headers: headers,
            body: body,
            values: HashMap::new(),
            files: HashMap::new(),
        };

        println!("\r\n\r\nStream ID: {}\r\n\r\n", response.stream_id().as_u32());

        let body = b"<h1 color=\"color: green;\">Hello World</h1>";

        // TODO: when make request from https and change http the request hangs mush check if it has change
        // if it has change must close the connection.
        let resp = H2Response::builder()
            .status(StatusCode::OK)
            .header("content-length", format!("{}", body.len()))
            .body(())
            .map_err(|e| IoError::new(ErrorKind::Other, format!("h2 build resp error: {e}")))?;

        let mut send = response
            .send_response(resp, false)
            .map_err(|e| IoError::new(ErrorKind::Other, format!("h2 send resp error: {e}")))?;

        send.send_data(Bytes::from_static(body), true)
            .map_err(|e| IoError::new(ErrorKind::Other, format!("h2 send data error: {e}")))?;

        Ok(())
    }

    fn hashmap_to_headers(&mut self, map: &HeaderMap) -> Headers {
        let mut headers = Headers::new();

        for (k, v) in map.iter() {
            headers.insert(
                k.as_str().to_string(),
                v.to_str().unwrap_or_default().to_string()
            );
        }

        return headers;
    }
}