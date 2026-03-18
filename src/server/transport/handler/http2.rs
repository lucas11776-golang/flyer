use std::io::Result;
use std::net::SocketAddr;

use bytes::Bytes;

use h2::server::SendResponse;
use http::Response as HttpResponse;
use url_domain_parse::Url;

use crate::{cookies::Cookies, request::{form::{Files, Form}}, utils::Headers};
use crate::response::{Response};
use crate::request::Request;
use crate::utils::url::parse_query_params;
use crate::utils::Values;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub(crate) struct Handler {
    addr: SocketAddr,
    send: SendResponse<bytes::Bytes>,
}

impl Handler {
    pub fn new(addr: SocketAddr, send: SendResponse<bytes::Bytes>) -> Self {
        return Self {
            addr: addr,
            send: send
        };
    }

    pub async fn handle(&mut self, request: http::Request<h2::RecvStream>,) -> Result<Request> {
        let mut headers = Headers::new();

        for (k, v) in request.headers().iter() {
            headers.insert(String::from(k.as_str().to_lowercase()), v.to_str().unwrap_or_default().to_string());
        }

        return Ok(Request {
            ip: self.addr.ip().to_string(),
            host: headers.get("host")
                .cloned()
                .or_else(|| headers.get(":authority").cloned())
                .or_else(|| Url::parse(request.uri().to_string().as_str()).unwrap().host().map(|v| v.to_string()))
                .unwrap_or_default(),
            method: request.method().to_string(),
            path: Url::parse(request.uri().to_string().as_str()).unwrap().path().to_string(),
            parameters: Values::new(),
            query: parse_query_params(request.uri().query().unwrap_or("")),
            protocol: "HTTP/2.0".to_string(),
            headers: headers,
            body: request.into_body().data().await.or(Some(Ok(Bytes::new()))).unwrap().unwrap().to_vec(),
            form: Form::new(Values::new(), Files::new()),
            session: None,
            cookies: Box::new(Cookies::new(Values::new())),
        });
    }

    pub async fn write(&mut self, req: &mut Request, res: &mut Response) -> Result<()> {
        let mut builder = HttpResponse::builder().status(res.status_code);

        for (k, v) in &mut res.headers {
            builder = builder.header(k.clone(), v.clone());
        }

        for cookie in &mut req.cookies.new_cookie {
            builder = builder.header("Set-Cookie", cookie.parse());
        }

        self.send.send_response(builder.body(()).unwrap(), false)
            .unwrap()
            .send_data(Bytes::from(res.body.clone()), true)
            .unwrap();

        Ok(())
    }
}