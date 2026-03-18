use serde::Serialize;

use crate::{utils::{Headers, Values},
    view::{ViewBag, ViewData},
    ws::Writer
};

pub const HTTP_CONTINUE:               u16 = 100;
pub const HTTP_SWITCHING_PROTOCOLS:    u16 = 101;
pub const HTTP_OK:                     u16 = 200;
pub const HTTP_CREATED:                u16 = 201;
pub const HTTP_ACCEPTED:               u16 = 202;
pub const HTTP_NO_CONTENT:             u16 = 204;
pub const HTTP_MOVED_PERMANENTLY:      u16 = 301;
pub const HTTP_FOUND:                  u16 = 302;
pub const HTTP_SEE_OTHER:              u16 = 303;
pub const HTTP_NOT_MODIFIED:           u16 = 304;
pub const HTTP_TEMPORARY_REDIRECT:     u16 = 307;
pub const HTTP_PERMANENT_REDIRECT:     u16 = 308;
pub const HTTP_BAD_REQUEST:            u16 = 400;
pub const HTTP_UNAUTHORIZED:           u16 = 401;
pub const HTTP_FORBIDDEN:              u16 = 403;
pub const HTTP_NOT_FOUND:              u16 = 404;
pub const HTTP_METHOD_NOT_ALLOWED:     u16 = 405;
pub const HTTP_CONFLICT:               u16 = 409;
pub const HTTP_GONE:                   u16 = 410;
pub const HTTP_TOO_MANY_REQUESTS:      u16 = 429;
pub const HTTP_INTERNAL_SERVER_ERROR:  u16 = 500;
pub const HTTP_NOT_IMPLEMENTED:        u16 = 501;
pub const HTTP_BAD_GATEWAY:            u16 = 502;
pub const HTTP_SERVICE_UNAVAILABLE:    u16 = 503;
pub const HTTP_GATEWAY_TIMEOUT:        u16 = 504;

pub struct Response {
    pub ws: Option<Box<dyn Writer + Send + Sync + 'static>>,
    pub(crate) status_code: u16,
    pub(crate) headers: Headers,
    pub(crate) referer: String,
    pub(crate) body: Vec<u8>,
    pub(crate) view: Option<ViewBag>,
    pub(crate) errors: Values,
    pub(crate) old: Values,
}

impl Response {
    pub fn new() -> Self {
        return Self {
            ws: None,
            status_code: HTTP_OK,
            headers: Headers::new(),
            referer: String::new(),
            body: vec![],
            view: None,
            errors: Values::new(),
            old: Values::new(),
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
            view: String::from(view),
            data: data
        });

        return self;
    }

    pub fn redirect(&mut self, path: &str) -> &mut Response {
        return self.html(&self.redirect_document(path)).status_code(HTTP_TEMPORARY_REDIRECT);
    }

    pub fn redirect_permanent(&mut self, path: &str) -> &mut Response {
        return self.html(&self.redirect_document(path)).status_code(HTTP_PERMANENT_REDIRECT);
    }

    fn redirect_document(&self, to: &str) -> String {
        return format!(r#"
        <!DOCTYPE html>
        <meta http-equiv="Refresh" content="0, url='{}'">
        <head>
            <body>
            </body>
        </html>
        "#, to);
    }

    pub fn back(&mut self) -> &mut Self {
        if self.referer.is_empty() {
            return self.redirect("/");
        }
        
        return self.redirect(&self.referer.clone());
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

    pub(crate) fn with_old(&mut self, old: Values) -> &mut Response {
        for (k, v) in old {
            self.old.insert(k, v);
        }

        return self;
    }
}