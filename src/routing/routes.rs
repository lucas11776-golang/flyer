use std::{collections::{HashMap, HashSet}};

use anyhow::Result;
use url_domain_parse::Url;

use crate::{
    error::logger::PanicErrorInfo,
    request::Request,
    response::{HTTP_INTERNAL_SERVER_ERROR, HTTP_NOT_FOUND, Response},
    routing::{HttpErrorHandler, HttpHandler, Middlewares, WebsocketHandler, next::Next, route::Route},
    utils::{Values, url}
};

pub struct Routes {
    pub(crate) http: Vec<Box<Route<HttpHandler>>>,
    pub(crate) websocket: Vec<Box<Route<WebsocketHandler>>>,
    pub(crate) middlewares: Middlewares, 
    pub(crate) errors: Vec<HttpErrorHandler>
}

impl Routes {
    pub fn new() -> Self {
        return Self {
            http: Vec::new(),
            websocket: Vec::new(),
            middlewares: HashMap::new(),
            errors: Vec::new(),
        };
    }

    pub async fn handle_http(&self, req: Request, res: Response) -> (Request, Response) {
        let (req, mut res, route) = self.handler(req.clone(), res, &self.http).await;

        if let None = route {
            res.status_code = HTTP_NOT_FOUND;
            
            return (req, res);
        }

        let res = (route.unwrap().handler)(req.clone(), res).await;

        return (req, res);
    }

    // pub async fn _handle_websocket<H>(&self, req: Request, res: Response, _routes: Vec<H>) -> Result<(Request, Response)> {
    //     return Ok((req, res));
    // }



    pub async fn handle_websocket(&self, req: Request, res: Response) -> (Request, Option<&Box<Route<WebsocketHandler>>>) {
        let (req, mut res, route) = self.handler(req.clone(), res, &self.websocket).await;

        if let None = route {
            res.status_code = HTTP_NOT_FOUND;
            
            return (req, None);
        }
        
        return (req, route);
    }

    pub async fn handle_error(&self, error: PanicErrorInfo, mut req: Request, mut res: Response) -> (Request, Response) {
        res.status_code = HTTP_INTERNAL_SERVER_ERROR;

        for callback in &self.errors {
            res.next(false);
            
            res = callback(error.clone(), req.clone(), res, Next::new()).await;

            if !res.is_next() {
                return (req, res);
            } 

            req = res.request();
        }

        return (req, res);
    }

    async fn resolve_middleware<'h, H>(&self, mut req: Request, mut res: Response, middlewares: HashSet<String>) -> (bool, Request, Response) {
        for middleware in middlewares {
            let callback = self.middlewares
                .get(&middleware)
                .unwrap()
                .as_ref();

            res.next(false);

            res = (callback)(req.clone(), res, Next::new()).await;

            if !res.is_next() {
                return (false, req, res);
            }

            req = res.request();
        }

        return (true, req, res);
    }

    async fn handler<'h, H>(&self, mut req: Request, res: Response, routes: &'h [Box<Route<H>>]) -> (Request, Response, Option<&'h Box<Route<H>>>) {
        for route in routes {
            if self.is_match(route, &mut req) {
                let (resolved, req, res) = self.resolve_middleware::<H>(req, res, route.middlewares.clone()).await;

                if !resolved {
                    return (req, res, None);
                }

                return (req, res, Some(route));
            }
        }

        return (req, res, None);
    }

    fn is_match<'a, H>(&self, route: &Route<H>, req: &mut Request) -> bool {
        if !route.method.eq_ignore_ascii_case(&req.method) {
            return false;
        }

        self.parameters_route_match(route, req)
    }
    
    fn parameters_route_match<H>(&self, route: &Route<H>, req: &mut Request) -> bool {
        let Some(url) = self.parse_request_url(&req.host) else {
            return false;
        };

        let mut parameters = Values::new();

        if !self.match_subdomains(&route.subdomain, &url, &mut parameters) {
            return false;
        }

        if !self.match_path_segments(&route.path, &req.path, &mut parameters) {
            return false;
        }

        req.parameters = parameters;

        return true;
    }

    fn parse_request_url(&self, host: &str) -> Option<Url> {
        let host_clean = host
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_start_matches("www");
        
        Url::parse(&format!("http://{}", host_clean)).ok()
    }

    fn match_subdomains(&self, route_subdomain: &str, url: &Url, parameters: &mut Values) -> bool {
        let route_subs: Vec<&str> = route_subdomain.split('.').filter(|s| !s.is_empty()).collect();
        let req_sub_str = url.subdomain().unwrap_or_default();
        let req_subs: Vec<&str> = req_sub_str.split('.').filter(|s| !s.is_empty()).collect();

        if route_subs.len() != req_subs.len() {
            return false;
        }

        for (r_sub, q_sub) in route_subs.iter().zip(req_subs.iter()) {
            if r_sub == q_sub {
                continue;
            }

            if let Some((k, v)) = self.dynamic_parameter_match(r_sub, q_sub) {
                parameters.insert(k, v);
            } else {
                return false;
            }
        }

        return true;
    }

    fn match_path_segments(&self, route_segments: &[String], req_path: &str, parameters: &mut Values) -> bool {
        let req_segments = url::clean(req_path);
        let has_wildcard = route_segments.last().map_or(false, |s| s == "*");
        
        if !has_wildcard && route_segments.len() != req_segments.len() {
            return false;
        }

        for (i, route_seg) in route_segments.iter().enumerate() {
            if route_seg == "*" {
                return true;
            }

            let Some(req_seg) = req_segments.get(i) else {
                return false;
            };

            if route_seg == req_seg {
                continue;
            }

            if let Some((k, v)) = self.dynamic_parameter_match(route_seg, req_seg) {
                parameters.insert(k, v);
            } else {
                return false;
            }
        }

        return true;
    }

    fn dynamic_parameter_match(&self, route_seg: &str, req_seg: &str) -> Option<(String, String)> {
        if route_seg.starts_with('{') && route_seg.ends_with('}') {
            let key = route_seg[1..route_seg.len() - 1].to_string();

            return Some((key, req_seg.to_string()));
        }

        return None;
    }
}