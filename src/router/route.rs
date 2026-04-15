use std::fmt::Debug;

use async_std::task::block_on;
use regex::Regex;
use url_domain_parse::Url;
use once_cell::sync::Lazy;

use crate::{
    request::Request,
    response::Response,
    router::{middleware::register, next::Next},
    utils::{Values, url::uri_to_segments}
};

const PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\{([a-zA-Z_]+)\}$").expect("Invalid parameter regex"));

pub struct Route<Handler: ?Sized> {
    pub(crate) subdomain: String,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) handler: Box<Handler>,
    pub(crate) middlewares: Vec<String>,
}

impl<Handler: ?Sized> Debug for Route<Handler> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("Route")
            .field("subdomain", &self.subdomain)
            .field("method", &self.method)
            .field("path", &self.path)
            .field("middlewares", &self.middlewares)
            .finish()
    }
}

impl <Handler: ?Sized>Route<Handler> {
    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static,
    {
        self.middlewares
            .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));

        return self;
    }

    pub(crate) fn is_match<'a>(&self, req: &'a mut Request) -> (bool, Values) {
        if self.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        return self.parameters_route_match(req);
    }
    
    pub fn parameters_route_match(&self, req: &Request) -> (bool, Values) {
        let mut parameters = Values::new();

        let host_clean = req.host.trim_start_matches("http://").trim_start_matches("https://");
        let Ok(url) = Url::parse(&format!("http://{}", host_clean)) else {
            return (false, Values::new());
        };
        let sub_route: Vec<&str> = self.subdomain.split('.').filter(|s| !s.is_empty()).collect();
        let sub_req_str = url.subdomain().unwrap_or_default();
        let sub_req: Vec<&str> = sub_req_str.split('.').filter(|s| !s.is_empty()).collect();

        if sub_route.len() != sub_req.len() {
            return (false, Values::new());
        }

        for (r_sub, q_sub) in sub_route.iter().zip(sub_req.iter()) {
            if r_sub == q_sub {
                continue;
            }

            if let Some((k, v)) = self.dynamic_parameter_match(r_sub, q_sub) {
                parameters.insert(k, v);
            } else {
                return (false, Values::new());
            }
        }

        let route_segments = uri_to_segments(self.path.clone());
        let req_segments = uri_to_segments(req.path.clone());
        let has_wildcard = route_segments.last() == Some(&String::from("*"));
        
        if !has_wildcard && route_segments.len() != req_segments.len() {
            return (false, Values::new());
        }

        for (i, req_seg) in route_segments.iter().enumerate() {
            if *req_seg == "*" {
                return (true, parameters);
            }

            let Some(q_seg) = req_segments.get(i) else {
                return (false, Values::new());
            };

            if req_seg == q_seg {
                continue;
            }

            if let Some((k, v)) = self.dynamic_parameter_match(req_seg, q_seg) {
                parameters.insert(k, v);
            } else {
                return (false, Values::new());
            }
        }

        return (true, parameters);
    }

    fn dynamic_parameter_match(&self, route_seg: &str, req_seg: &str) -> Option<(String, String)> {
        return PARAM_REGEX.captures(route_seg).map(|cap| (cap[1].to_string(), req_seg.to_string()));
    }
}