use std::{io::Result};

use serde::Serialize;

use crate::{
    request::Headers,
    session::Session,
    ws::Ws,
    view::{
        View,
        ViewData
    }
};

pub struct Response {
    pub(crate) status_code: u16,
    pub(crate) headers: Headers,
    pub(crate) body: Vec<u8>,
    pub(crate) session: Option<Box<dyn Session>>,
    pub(crate) view: Option<View>,
    pub ws: Option<Ws>,
}

pub fn new_response() -> Response {
    return Response {
        status_code: 200,
        headers: Headers::new(),
        body: vec![],
        session: None,
        view: None,
        ws: None
    };
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
    pub fn status_code(&mut self, code: u16) -> &mut Response {
        self.status_code = code;
        
        return self;
    }

    pub fn header(&mut self, key: String, value: String) -> &mut Response {
        self.headers.insert(key, value);

        return self;
    }

    pub fn headers(&mut self, headers: Headers) -> &mut Response {
        for ele in headers {
            self.header(ele.0, ele.1);
        }

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
        let html = self.view.as_mut().unwrap().render(view, data);

        return self.html(&html);
    }

    pub fn session<'a>(&self) -> Option<&Box<dyn Session>> {
        return self.session.as_ref();
    }

    pub fn clone(&self) -> Response {
        return Response {
            status_code: self.status_code,
            headers: self.headers.clone(),
            body: self.body.clone(),
            session: None,
            view: None,
            ws: None
        };
    }
}



