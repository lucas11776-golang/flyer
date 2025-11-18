use std::io::Result;

use bytes::Bytes;

use http::Request as HttpRequest;
use h3::server::RequestStream;
use h3_quinn::BidiStream;

use crate::cookie::Cookies;
use crate::request::Request;
use crate::request::form::Form;
use crate::request::parse::parse_content_type;
use crate::response::Response;
use crate::server::HTTP3;
use crate::utils::url::parse_query_params;
use crate::utils::Values;

pub(crate) struct Handler {
    request: HttpRequest<()>,
    stream: RequestStream<BidiStream<Bytes>, Bytes>
}

impl Handler {
    pub fn new(request: HttpRequest<()>, stream: RequestStream<BidiStream<Bytes>, Bytes>) -> Self {
        return Self {
            request: request,
            stream: stream
        }
    }

    fn get_headers(&mut self) -> Values {
        let mut headers = Values::new();

        for (k, v) in self.request.headers() {
            headers.insert(k.to_string(), v.to_str().unwrap().to_string());
        }

        return headers;
    }

    fn get_host(&mut self, headers: &Values) -> String {
        return headers
            .get("host")
            .cloned()
            .or_else(|| headers.get(":authority").cloned())
            .unwrap_or_default();
    }

    pub async fn handle(&mut self) -> Result<Request> {
        let headers = self.get_headers();
        
        let req = Request{
            ip: "127.0.0.1".to_owned(),
            host: self.get_host(&headers),
            headers: headers,
            method: self.request.method().to_string(),
            path: self.request.uri().path().to_string(),
            parameters: Values::new(),
            query: parse_query_params(self.request.uri().query().unwrap_or(""))?,
            protocol: HTTP3.to_string(),
            body: vec![],
            form: Form::new(),
            session: None,
            cookies: Box::new(Cookies::new(Values::new())),
        };

        return Ok(parse_content_type(req).await.unwrap());
    }

    pub async fn write(mut self, req: &mut Request, res: &mut Response) -> Result<()> {
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

