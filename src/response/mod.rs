pub mod parser;

use serde::Serialize;

use crate::{
    utils::{Headers, Values},
    view::ViewData,
    ws::Writer
};

pub struct Response {
    pub ws: Option<Box<dyn Writer + Send + Sync + 'static>>,
    pub(crate) status_code: u16,
    pub(crate) headers: Headers,
    pub(crate) request_headers: Headers,
    pub(crate) body: Vec<u8>,
    pub(crate) view: Option<ViewBag>,
    pub(crate) errors: Values,
}

#[derive(Clone)]
pub struct ViewBag {
    pub(crate) view: String,
    pub(crate) data: Option<ViewData>,
}

impl Response {
    pub fn new() -> Self {
        return Self {
            ws: None,
            status_code: 200,
            headers: Headers::new(),
            request_headers: Headers::new(),
            body: vec![],
            view: None,
            errors: Values::new(),
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

    pub fn back(&mut self) -> &mut Self {
        let redirect = self.request_headers.get("referer");

        if redirect.is_none() {
            return self.redirect("/");
        }

        return self.redirect(&redirect.unwrap().clone());
    }

    pub fn with_error(&mut self, name: &str, error: &str) -> &mut Response {
        self.errors.insert(name.to_string(), error.to_string());

        return self;
    }

    pub fn with_errors(&mut self, errors: Values) -> &mut Response {
        for (name, error) in errors {
            self.with_error(name.as_str(), error.as_str());
        }

        return self;
    }
}