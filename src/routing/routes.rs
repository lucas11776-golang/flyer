use std::collections::{HashMap, HashSet};

use crate::{
    error::logger::PanicErrorInfo,
    request::Request,
    response::{Response, HTTP_INTERNAL_SERVER_ERROR, HTTP_NOT_FOUND},
    routing::{
        next::Next, route::Route, HttpErrorHandler, HttpHandler, Middlewares, WebsocketHandler,
    },
    utils::{url, Values},
};

struct RequestContext<'a> {
    subdomains: Vec<&'a str>,
    path_segments: Vec<String>,
}

impl<'a> RequestContext<'a> {
    fn from_request(req: &'a Request) -> Self {
        let host_clean = req
            .host
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or("")
            .trim_start_matches("www.");

        let subdomains = host_clean
            .split('.')
            .filter(|s| !s.is_empty())
            .collect();

        let path_segments = url::clean(&req.path);

        Self {
            subdomains,
            path_segments,
        }
    }
}

pub struct Routes {
    pub(crate) http: Vec<Route<HttpHandler>>,
    pub(crate) websocket: Vec<Route<WebsocketHandler>>,
    pub(crate) middlewares: Middlewares,
    pub(crate) errors: Vec<HttpErrorHandler>,
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
        }
    }

    pub async fn handle_http(&self, req: Request, res: Response) -> (Request, Response) {
        let (req, mut res, route) = self.handler(req, res, &self.http).await;

        let Some(route) = route else {
            res.status_code = HTTP_NOT_FOUND;
            return (req, res);
        };

        let res = (route.handler)(req.clone(), res).await;
        (req, res)
    }

    pub async fn handle_websocket(&self,
        req: Request,
        res: Response,
    ) -> (Request, Option<&Route<WebsocketHandler>>) {
        let (req, mut res, route) = self.handler(req, res, &self.websocket).await;

        if route.is_none() {
            res.status_code = HTTP_NOT_FOUND;
            return (req, None);
        }

        (req, route)
    }

    pub async fn handle_error(
        &self,
        error: PanicErrorInfo,
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
            if let Some(callback) = self.middlewares.get(middleware) {
                res.next(false);
                res = (callback.as_ref())(req.clone(), res, Next::new()).await;

                if !res.is_next() {
                    return (false, req, res);
                }

                req = res.request();
            }
        }

        (true, req, res)
    }

    async fn handler<'h, H>(
        &'h self,
        mut req: Request,
        res: Response,
        routes: &'h [Route<H>],
    ) -> (Request, Response, Option<&'h Route<H>>) {
        let ctx = RequestContext::from_request(&req);

        for route in routes {
            if let Some(params) = self.match_route(route, &req.method, &ctx) {
                req.parameters = params;

                let (resolved, req, res) = self
                    .resolve_middleware(req, res, &route.middlewares)
                    .await;

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
        ctx: &RequestContext,
    ) -> Option<Values> {
        if !route.method.eq_ignore_ascii_case(req_method) {
            return None;
        }

        let route_subs: Vec<&str> = route
            .subdomain
            .split('.')
            .filter(|s| !s.is_empty())
            .collect();

        if route_subs.len() != ctx.subdomains.len() {
            return None;
        }

        for (r_sub, q_sub) in route_subs.iter().zip(&ctx.subdomains) {
            if r_sub != q_sub && !is_param(r_sub) {
                return None;
            }
        }

        let has_wildcard = route.path.last().map_or(false, |s| s == "*");

        if !has_wildcard && route.path.len() != ctx.path_segments.len() {
            return None;
        }

        for (route_seg, req_seg) in route.path.iter().zip(&ctx.path_segments) {
            if route_seg == "*" {
                break;
            }
            if route_seg != req_seg && !is_param(route_seg) {
                return None;
            }
        }

        let mut parameters = Values::new();

        for (r_sub, q_sub) in route_subs.iter().zip(&ctx.subdomains) {
            if is_param(r_sub) {
                parameters.insert(r_sub[1..r_sub.len() - 1].to_string(), q_sub.to_string());
            }
        }

        for (route_seg, req_seg) in route.path.iter().zip(&ctx.path_segments) {
            if route_seg == "*" {
                break;
            }
            if is_param(route_seg) {
                parameters.insert(
                    route_seg[1..route_seg.len() - 1].to_string(),
                    req_seg.clone(),
                );
            }
        }

        Some(parameters)
    }
}

#[inline(always)]
fn is_param(seg: &str) -> bool {
    seg.starts_with('{') && seg.ends_with('}') && seg.len() > 2
}