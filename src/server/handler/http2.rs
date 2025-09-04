use bytes::Bytes;
use h2::{server, server::{SendResponse}};
use http::{HeaderMap, Request as HttpRequest, Response as HttpResponse};
use reqwest::Url;
use std::{collections::HashMap, io::Result};
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{response::{new_response}, server::handler::RequestHandler, HTTP};
use crate::request::{Headers, Request};
use crate::utils::url::parse_query_params;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub struct Handler<'a> {
    http: &'a mut HTTP,
    addr: SocketAddr,
}

impl <'a> Handler<'a> {
    pub async fn handle<RW>(http: &'a mut HTTP, rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + std::marker::Send
    {
        let mut handler: Handler<'_> = Handler{
            http: http,
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
                    let _ = self.new_request( req, response).await;
                });
            });
        }
    }

    async fn new_request(&mut self , request: HttpRequest<h2::RecvStream>, send: SendResponse<Bytes>) -> std::io::Result<()> {
        let method = request.method().to_string();
        let path = Url::parse(request.uri().to_string().as_str()).unwrap().path().to_string();
        let query = parse_query_params(request.uri().query().unwrap_or(""));
        let mut body = Vec::new();
        let headers = self.hashmap_to_headers(request.headers());
        let mut recv = request.into_body();

        while let Some(chunk) = recv.data().await.transpose().unwrap() {
            body.extend_from_slice(&chunk);
        }

        let host = headers
            .get("host")
            .cloned()
            .or_else(|| headers.get(":authority").cloned())
            .unwrap_or_default();

        let req = Request {
            ip: self.addr.ip().to_string(),
            host: host,
            method: method,
            path: path,
            query: query,
            protocol: "HTTP/2.0".to_string(),
            headers: headers,
            body: body,
            values: HashMap::new(),
            files: HashMap::new(),
        };

        self.handle_request(req, send).await?;

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

    async fn handle_request(&mut self, mut req: Request, mut send:  SendResponse<Bytes>) -> Result<()> {
        let mut response = new_response();
        let response = RequestHandler::web(&mut self.http, &mut req, &mut response).await?;

        let mut builder = HttpResponse::builder().status(response.status_code);

        for (k, v) in &mut response.headers {
            builder = builder.header(k.clone(), v.clone());
        }

        return Ok(
            send.send_response(builder.body(()).unwrap(), false)
                .unwrap()
                .send_data(Bytes::from(response.body.clone()), true)
                .unwrap()
        )
    }
}