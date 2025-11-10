use std::io::Result;
use serde::Serialize;

use crate::{
    request::Headers,
    view::ViewData,
    ws::{Writer},
};

pub struct Response {
    // TODO: should thing about give writer only because WS will be handled in controller...
    pub ws: Option<Box<dyn Writer + Send + Sync + 'static>>,
    pub(crate) status_code: u16,
    pub(crate) headers: Headers,
    pub(crate) body: Vec<u8>,
    pub(crate) view: Option<ViewBag>,
}

#[derive(Clone)]
pub struct ViewBag {
    pub(crate) view: String,
    pub(crate) data: Option<ViewData>,
}

pub fn parse(response: &mut Response) -> Result<String> {
    let mut res: Vec<String> = vec![format!("HTTP/1.0 {} {}", response.status_code, "OK")];

    for (k, v) in response.headers.clone() {
        res.push(format!("{}: {}", k, v));
    }

    res.push(format!("Content-Length: {}", response.body.len()));
    res.push(format!("\r\n{}", String::from_utf8(response.body.clone()).unwrap()));

    return Ok(res.join("\r\n"));
}

impl Response {
    pub fn new() -> Self {
        return Self {
            ws: None,
            status_code: 200,
            headers: Headers::new(),
            body: vec![],
            view: None,
        };
    }

    pub fn status_code(&mut self, code: u16) -> &mut Response {
        self.status_code = code;
        
        return self;
    }

    pub fn header(&mut self, key: String, value: String) -> &mut Response {
        self.headers.insert(key, value);

        return self;
    }

    pub fn headers(&mut self, headers: Headers) -> &mut Response {
        self.headers.extend(headers);

        return self;
    }

    pub fn body(&mut self, body: &[u8]) -> &mut Response {
        self.body = body.to_vec();

        return self;
    }

    pub fn json<J>(&mut self, object: &J) -> &mut Response
    where 
        J: ?Sized + Serialize
    {
        return self.header("Content-Type".to_owned(), "application/json".to_owned())
            .body(serde_json::to_string(object).unwrap().as_bytes());
    }

    pub fn html(&mut self, html: &str) -> &mut Response {
        return self.header("Content-Type".to_string(), "text/html".to_owned())
            .body(html.as_bytes());
    }

    pub fn view(&mut self, view: &str, data: Option<ViewData>) -> &mut Response {
        self.view = Some(ViewBag {
            view: view.to_string(),
            data: data
        });

        return self;
    }
}

