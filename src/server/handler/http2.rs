use std::{collections::HashMap, io::Result};
use std::net::SocketAddr;
use std::pin::Pin;

use bytes::Bytes;

use h2::{server, server::{SendResponse}};
use http::{HeaderMap, Request as HttpRequest, Response as HttpResponse};
use reqwest::Url;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::response::{Response};
use crate::request::{Headers, Request};
use crate::utils::url::parse_query_params;
use crate::utils::Values;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub(crate) struct Handler<'a, RW> {
    addr: SocketAddr,
    conn: Box<server::Connection< Pin<&'a mut BufReader<RW>>, Bytes>>,
}

impl <'a, RW>Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    pub async fn new(addr: SocketAddr, rw: Pin<&'a mut BufReader<RW>>) -> Self {
        return Self {
            addr: addr,
            conn: Box::new(server::handshake(rw).await.unwrap()),
        };
    }
    
    pub async fn handle(&mut self) -> Option<Result<(HttpRequest<h2::RecvStream> , SendResponse<Bytes>)>> {
        while let Some(result) = self.conn.accept().await {
            return Some(Ok(result.unwrap()))
        }

        return None;
    }

    pub async fn write(&mut self, mut send: SendResponse<Bytes>, res: &mut Response) -> Result<()> {
        let mut builder = HttpResponse::builder().status(res.status_code);

        for (k, v) in &mut res.headers {
            builder = builder.header(k.clone(), v.clone());
        }

        send.send_response(builder.body(()).unwrap(), false)
            .unwrap()
            .send_data(Bytes::from(res.body.clone()), true)
            .unwrap();

        Ok(())
    }

    pub async fn get_http_request(&mut self, request: HttpRequest<h2::RecvStream>) -> Result<Request> {
        let method = request.method().to_string();
        let path = Url::parse(request.uri().to_string().as_str()).unwrap().path().to_string();
        let query = parse_query_params(request.uri().query().unwrap_or(""))?;
        let mut body = Vec::new();
        let headers = self.hashmap_to_headers(request.headers());
        let mut recv = request.into_body();
        let host = headers
            .get("host")
            .cloned()
            .or_else(|| headers.get(":authority").cloned())
            .unwrap_or_default();

        while let Some(chunk) = recv.data().await.transpose().unwrap() {
            body.extend_from_slice(&chunk);
        }

        Ok(Request {
            ip: self.addr.ip().to_string(),
            host: host,
            method: method,
            path: path,
            parameters: Values::new(),
            query: query,
            protocol: "HTTP/2.0".to_string(),
            headers: headers,
            body: body,
            values: HashMap::new(),
            files: HashMap::new(),
        })
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