use std::io::Result;
use std::net::SocketAddr;

use bytes::Bytes;

use http::Request as HttpRequest;
use h3::server::RequestStream;
use h3_quinn::BidiStream;

use crate::cookies::Cookies;
use crate::request::Request;
use crate::request::form::{Files, Form};
use crate::response::Response;
use crate::utils::url::parse_query_params;
use crate::utils::Values;

pub(crate) struct Handler {
    request: HttpRequest<()>,
    stream: RequestStream<BidiStream<Bytes>, Bytes>,
    addr: SocketAddr
}

impl Handler {
    pub fn new(request: HttpRequest<()>, stream: RequestStream<BidiStream<Bytes>, Bytes>, add: SocketAddr) -> Self {
        return Self {
            request: request,
            stream: stream,
            addr: add
        }
    }
    
    pub async fn handle(&mut self) -> Result<Request> {
        let mut headers = Values::new();

        for (k, v) in self.request.headers() {
            headers.insert(k.to_string(), v.to_str().unwrap().to_string());
        }
        
        return Ok(Request{
            ip: self.addr.to_string(),
            host: self.get_host(),
            headers: headers,
            method: self.request.method().to_string().to_uppercase(),
            path: self.request.uri().path().to_string(),
            parameters: Values::new(),
            query: parse_query_params(self.request.uri().query().unwrap_or("")),
            protocol: "HTTP/3.0".to_string(),
            body: vec![],
            form: Form::new(Values::new(), Files::new()),
            session: None,
            cookies: Box::new(Cookies::new(Values::new())),
        });
    }

    fn get_host(&mut self) -> String {
        return self.request.uri()
            .authority()
            .map(|a| String::from(a.as_str()))
            .or(self.request
                .headers()
                .get(http::header::HOST)
                .and_then(|h| Some(String::from(h.to_str().unwrap()))))
            .unwrap_or_default();
    }

    pub async fn write(&mut self, req: &mut Request, res: &mut Response) -> Result<()> {
        let mut builder = http::Response::builder()
            .status(res.status_code)
            .header("content-length", format!("{}", res.body.len()));

        for (k, v) in &mut res.headers {
            builder = builder.header(k.clone(), v.clone());
        }

        for cookie in &mut req.cookies.new_cookie {
            builder = builder.header("Set-Cookie", cookie.parse());
        }

        let response = builder.body(()).unwrap();

        self.stream.send_response(response).await.unwrap();
        self.stream.send_data(Bytes::from(res.body.to_owned()).clone()).await.unwrap();
        self.stream.finish().await.unwrap();

        Ok(())
    }
}

