use std::time::Duration;

use serde::Serialize;

use crate::{
    cookies::cookie::Cookie,
    hooks::Hook,
    request::Request,
    response::Response,
    routing::next::Next,
    utils::{Values, http::Headers}
};

pub mod cookie;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Cookies {
    inner: Values,
    outer: Vec<Cookie>,
}

impl Cookies {
    pub fn new() -> Self {
        return Self::default();
    }
}

impl Hook for Cookies {
    async fn before(&self, mut req: Request, res: Response, next: Next) -> Response {
        req.cookies.inner = self.parse(&req.header("cookie").into_boxed_str());

        return next.handle(req, res);
    }
    
    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        let mut headers = Headers::new();

        for cookie in &res.cookies.outer {
            headers.insert(cookie.name.clone(), cookie.parse());
        }

        return next.handle(req, res.set_headers(headers));
    }
}

impl Cookies {
    pub fn cookies(&self) -> Values {
        return self
            .inner
            .clone();
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        return self
            .inner
            .get(k)
            .map(|s| s.as_str())
    }

    pub fn set(&mut self, k: impl Into<String>, v: impl Into<String>) -> &mut Cookie {
        self
            .outer
            .push(Cookie::new(k, v));

        return self
            .outer
            .last_mut()
            .unwrap();
    } 

    pub fn remove(&mut self, k: impl Into<String>) {
        self
            .set(k, "")
            .set_expires(Duration::from_secs(0) - Duration::from_secs(100));
    }

    pub(crate) fn parse(&self, raw: &str) -> Values {
        let mut cookies = Values::new();

        ::cookie::Cookie::split_parse(raw)
            .filter_map(Result::ok)
            .for_each(|cookie| {
                cookies.insert(cookie.name().into(), cookie.value().into());
            });

        return cookies;
    }
}