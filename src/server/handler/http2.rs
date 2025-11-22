use std::io::Result;
use std::net::SocketAddr;

use bytes::Bytes;

use h2::{server, server::{SendResponse}};
use http::{HeaderMap, Request as HttpRequest, Response as HttpResponse};
use reqwest::Url;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{cookie::Cookies, request::{form::Form, parser::parse_content_type}, utils::Headers};
use crate::response::{Response};
use crate::request::Request;
use crate::utils::url::parse_query_params;
use crate::utils::Values;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub(crate) struct Handler<RW> {
    addr: SocketAddr,
    conn: Box<server::Connection< BufReader<RW>, Bytes>>,
}

impl <'a, RW>Handler<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    pub async fn new(addr: SocketAddr, rw: BufReader<RW>) -> Self {
        return Self {
            addr: addr,
            conn: Box::new(server::handshake(rw).await.unwrap()),
        };
    }
    
    pub async fn handle(&mut self) -> Option<std::result::Result<(HttpRequest<h2::RecvStream>, SendResponse<Bytes>), h2::Error>>  {
        // TODO: connection hugs when Switching from HTTP2 -> HTTP1
        return self.conn.accept().await;
    }

    pub async fn write(&mut self, mut send: SendResponse<Bytes>, req: &mut Request, res: &mut Response) -> Result<()> {
        let mut builder = HttpResponse::builder().status(res.status_code);

        for (k, v) in &mut res.headers {
            builder = builder.header(k.clone(), v.clone());
        }

        for cookie in &mut req.cookies.new_cookie {
            builder = builder.header("Set-Cookie", cookie.parse());
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
        let headers = self.hashmap_to_headers(request.headers());
        let host = headers.get("host").cloned().or_else(|| headers.get(":authority").cloned()).unwrap_or_default();
        let body = request.into_body().data().await.or(Some(Ok(Bytes::new()))).unwrap().unwrap().to_vec();

        let mut req = Request {
            ip: self.addr.ip().to_string(),
            host: host,
            method: method.to_uppercase(),
            path: path,
            parameters: Values::new(),
            query: query,
            protocol: "HTTP/2.0".to_string(),
            headers: headers,
            body: body,
            form: Form::new(),
            session: None,
            cookies: Box::new(Cookies::new(Values::new())),
        };

        if req.method == "POST" || req.method == "PATCH" || req.method == "PUT" {
            req = parse_content_type(req).await.unwrap();
        }

        return Ok(req);
    }

    fn hashmap_to_headers(&mut self, map: &HeaderMap) -> Headers {
        let mut headers = Headers::new();

        for (k, v) in map.iter() {
            headers.insert(k.as_str().to_string(), v.to_str().unwrap_or_default().to_string());
        }

        return headers;
    }
}