use std::io::Result;
use serde::Serialize;

use crate::{
    cookie::Cookie, request::Headers, view::ViewData, ws::Writer
};

pub struct Response {
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

pub fn parse(response: &mut Response, cookies: Option<&mut Vec<Cookie>>) -> Result<String> {
    let mut res: Vec<String> = vec![format!("HTTP/1.0 {} {}", response.status_code, "OK")];

    for (k, v) in response.headers.clone() {
        res.push(format!("{}: {}", k, v));
    }

    res.push(format!("Content-Length: {}", response.body.len()));
    
    if let Some(cookies) = cookies {
        for cookie in cookies {
            res.push(format!("Set-Cookie: {}", cookie.parse()));
        }
    }

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

    pub fn header(&mut self, key: &str, value: &str) -> &mut Response {
        self.headers.insert(key.to_owned(), value.to_owned());

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
        return self.header("Content-Type", "application/json")
            .body(serde_json::to_string(object).unwrap().as_bytes());
    }

    pub fn html(&mut self, html: &str) -> &mut Response {
        return self.header("Content-Type", "text/html")
            .body(html.as_bytes());
    }

    pub fn view(&mut self, view: &str, data: Option<ViewData>) -> &mut Response {
        self.view = Some(ViewBag {
            view: view.to_string(),
            data: data
        });

        return self;
    }

    pub fn redirect(&mut self, to: &str) -> &mut Response {
        let html = format!(r#"
        <!DOCTYPE html>
        <meta http-equiv="Refresh" content="0, url='{}'">
        <head>
            <body>
            </body>
        </html>
        "#, to);

        return self.html(&html).status_code(307);
    }
}