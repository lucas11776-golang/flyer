use std::{
    any::Any,
    collections::HashMap,
    io::Result
};

use serde::Serialize;
use tera::{to_value, Context, Tera};

use crate::{
    request::Headers, session::Session, view::View, Configuration
};

#[derive(Debug)]
pub struct Response {
    pub(crate) status_code: u16,
    pub(crate) headers: Headers,
    pub(crate) body: Vec<u8>,
    pub(crate) session: Option<Box<dyn Session>>,
    pub(crate) config: Configuration,
}

pub fn new_response() -> Response {
    return Response {
        status_code: 200,
        headers: Headers::new(),
        body: vec![],
        session: None,
        config: Configuration::new(),
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

    // where -> specify the type of data that is allowed in T.
    pub fn json<T>(&mut self, json: &T) -> &mut Response
    where T: ?Sized + Serialize
    {
        return self.header("Content-Type".to_owned(), "application/json".to_owned())
            .body(serde_json::to_string(json).unwrap().as_bytes());
    }

    pub fn html(&mut self, html: &str) -> &mut Response {
        return self.header("Content-Type".to_string(), "text/html".to_owned())
            .body(html.as_bytes());
    }

    pub fn view(&mut self, name: &str, data: Option<ViewData>) -> &mut Response {
        // TODO: move to HTTP....
        let tera = Tera::new("views/**/*").unwrap();

        let context: Context;

        match data {
            Some(ctx) => context = ctx.context,
            None => context = Context::new(),
        }

        let html = tera.render(&format!("{}.html", name), &context).unwrap();

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
            config: Configuration::new(),
        };
    }
}



// TODO: move to view namespace
pub fn view_data() -> ViewData {
    return ViewData{
        context: Context::new()
    };
}

pub struct ViewData {
    pub(crate) context: Context, 
}

impl ViewData {
     pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.context.insert(key, val);

    }
}
