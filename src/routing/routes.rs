use std::collections::{HashMap, HashSet};
use url_domain_parse::Url;

use crate::{
    error::Error,
    request::Request,
    response::{HTTP_INTERNAL_SERVER_ERROR, HTTP_NOT_FOUND, Response},
    routing::{
        HttpErrorHandler, HttpHandler, Middlewares, WebsocketHandler, next::Next, route::Route,
    },
    utils::{Values, url},
};

pub struct Routes {
    pub(crate) http: Vec<Route<HttpHandler>>,
    pub(crate) websocket: Vec<Route<WebsocketHandler>>,
    pub(crate) middlewares: Middlewares,
    pub(crate) errors: Vec<HttpErrorHandler>,
    pub(crate) not_found_callback: Option<HttpHandler>,
}

impl Default for Routes {
    fn default() -> Self {
        Self::new()
    }
}

impl Routes {
    pub fn new() -> Self {
        Self {
            http: Vec::new(),
            websocket: Vec::new(),
            middlewares: HashMap::new(),
            errors: Vec::new(),
            not_found_callback: None,
        }
    }

    pub async fn handle_http(&self, req: Request, res: Response) -> (Request, Response) {
        let (req, mut res, route) = self.handler(req, res, &self.http).await;

        let Some(route) = route else {
            res.status_code = HTTP_NOT_FOUND;

            if let Some(cb) = &self.not_found_callback {
                res = cb(req.clone(), res).await;
            }

            return (req, res);
        };

        let res = (route.handler)(req.clone(), res).await;

        (req, res)
    }

    pub async fn handle_websocket(
        &self,
        req: Request,
        res: Response,
    ) -> (Request, Response, Option<&Route<WebsocketHandler>>) {
        let (req, mut res, route) = self.handler(req, res, &self.websocket).await;

        if route.is_none() {
            res.status_code = HTTP_NOT_FOUND;
            return (req, res, None);
        }

        (req, res, route)
    }

    pub async fn handle_error(
        &self,
        error: Error,
        mut req: Request,
        mut res: Response,
    ) -> (Request, Response) {
        res.status_code = HTTP_INTERNAL_SERVER_ERROR;

        for callback in &self.errors {
            res.next(false);
            res = callback(error.clone(), req.clone(), res, Next::new()).await;

            if !res.is_next() {
                return (req, res);
            }

            req = res.request();
        }

        (req, res)
    }

    async fn resolve_middleware(
        &self,
        mut req: Request,
        mut res: Response,
        middlewares: &HashSet<String>,
    ) -> (bool, Request, Response) {
        for middleware in middlewares {
            let Some(callback) = self.middlewares.get(middleware) else {
                continue;
            };

            res.next(false);
            res = (callback.as_ref())(req.clone(), res, Next::new()).await;

            if !res.is_next() {
                return (false, req, res);
            }

            req = res.request();
        }

        (true, req, res)
    }

    async fn handler<'h, H>(
        &self,
        mut req: Request,
        res: Response,
        routes: &'h [Route<H>],
    ) -> (Request, Response, Option<&'h Route<H>>) {
        let req_segments = url::clean(&req.path);
        let parsed_url = self.parse_request_url(&req.host);

        for route in routes {
            if let Some(params) =
                self.match_route(route, &req.method, &req_segments, parsed_url.as_ref())
            {
                req.parameters = params;

                let (resolved, req, res) =
                    self.resolve_middleware(req, res, &route.middlewares).await;

                if !resolved {
                    return (req, res, None);
                }

                return (req, res, Some(route));
            }
        }

        (req, res, None)
    }

    fn match_route<H>(
        &self,
        route: &Route<H>,
        req_method: &str,
        req_segments: &[String],
        parsed_url: Option<&Url>,
    ) -> Option<Values> {
        if !route.method.eq_ignore_ascii_case(req_method) {
            return None;
        }

        let url = parsed_url?;
        let req_sub_str = url.subdomain().unwrap_or_default();

        let mut extracted_params = Vec::with_capacity(4);

        if !self.match_subdomains(&route.subdomain, &req_sub_str, &mut extracted_params) {
            return None;
        }

        if !self.match_path_segments(&route.path, req_segments, &mut extracted_params) {
            return None;
        }

        let mut parameters = Values::new();
        for (k, v) in extracted_params {
            parameters.insert(k.to_string(), v.to_string());
        }

        Some(parameters)
    }

    fn parse_request_url(&self, host: &str) -> Option<Url> {
        if host.starts_with("http://") || host.starts_with("https://") {
            Url::parse(host).ok()
        } else {
            let host_clean = host.trim_start_matches("www.");
            let mut url_buf = String::with_capacity(7 + host_clean.len());
            url_buf.push_str("http://");
            url_buf.push_str(host_clean);
            Url::parse(&url_buf).ok()
        }
    }

    fn match_subdomains<'a>(
        &self,
        route_subdomain: &'a str,
        req_sub_str: &'a str,
        parameters: &mut Vec<(&'a str, &'a str)>,
    ) -> bool {
        let mut route_subs = route_subdomain.split('.').filter(|s| !s.is_empty());
        let mut req_subs = req_sub_str.split('.').filter(|s| !s.is_empty());

        loop {
            match (route_subs.next(), req_subs.next()) {
                (Some(r_sub), Some(q_sub)) => {
                    if r_sub == q_sub {
                        continue;
                    }
                    if let Some((k, v)) = self.dynamic_parameter_match(r_sub, q_sub) {
                        parameters.push((k, v));
                    } else {
                        return false;
                    }
                }
                (None, None) => return true,
                _ => return false,
            }
        }
    }

    fn match_path_segments<'a>(
        &self,
        route_segments: &'a [String],
        req_segments: &'a [String],
        parameters: &mut Vec<(&'a str, &'a str)>,
    ) -> bool {
        let has_wildcard = route_segments.last().map_or(false, |s| s == "*");

        if !has_wildcard && route_segments.len() != req_segments.len() {
            return false;
        }

        for (route_seg, req_seg) in route_segments.iter().zip(req_segments.iter()) {
            if route_seg == "*" {
                return true;
            }
            if route_seg == req_seg {
                continue;
            }
            if let Some((k, v)) = self.dynamic_parameter_match(route_seg, req_seg) {
                parameters.push((k, v));
            } else {
                return false;
            }
        }

        true
    }

    #[inline]
    fn dynamic_parameter_match<'a>(
        &self,
        route_seg: &'a str,
        req_seg: &'a str,
    ) -> Option<(&'a str, &'a str)> {
        if route_seg.starts_with('{') && route_seg.ends_with('}') && route_seg.len() > 2 {
            return Some((&route_seg[1..route_seg.len() - 1], req_seg));
        }
        None
    }
}
