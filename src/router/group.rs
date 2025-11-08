use std::{collections::HashMap, io::Result};
use regex::Regex;
use once_cell::sync::Lazy;

use futures::executor::block_on;

use crate::{
    request::Request,
    response::Response,
    router::{Middlewares, MiddlewaresRef, Next, Route, WebRoute, WsRoute},
    utils::{Values, url::clean_uri_to_vec},
};

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex")
});


#[derive(Default)]
pub struct GroupRouter {
    pub(crate) web: Vec<Route<Box<WebRoute>>>,
    pub(crate) ws: Vec<Route<Box<WsRoute>>>,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
    pub(crate) middlewares: Middlewares,
}

impl <'r>GroupRouter {
    pub fn new() -> Self {
        return GroupRouter {
            web: vec![],
            ws: vec![],
            not_found_callback: None,
            middlewares: HashMap::new(),
        }
    }

    pub fn add_web_route<C>(&mut self, method: &str, path: String, callback: C, middlewares: MiddlewaresRef)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.web.push(Route{
            path: path,
            method: method.to_string(),
            route: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: middlewares,
        });
    }

    pub async fn match_web_routes(&mut self, req: &'r mut Request, res: &'r mut Response) -> Result<&'r mut Response> {
        for route in &mut self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if Self::handle_middlewares(&self.middlewares, req, res, &route.middlewares).is_none() {
                return Ok(res)
            }

            return Ok((route.route)(req, res))
        }

        if self.not_found_callback.is_some() {
            return Ok(self.not_found_callback.as_ref() .unwrap()(req, res));
        }

        res.status_code = 404;

        return Ok(res)
    }

    pub async fn match_ws_routes(&mut self, req: &'r mut Request, res: &'r mut Response) -> Result<bool> {
        for route in &mut self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if Self::handle_middlewares(&self.middlewares, req, res, &route.middlewares).is_none() {
                return Ok(false)
            }

            let (ws, _) = res.ws.as_mut().unwrap();

            (route.route)(req, ws);

            return Ok(true);
        }

        return Ok(false);
    }

    // fn match_route<T>(route: &mut Route<T>, req: &mut Request) -> (bool, Values) {
    //     let request_path: Vec<String> = clean_uri_to_vec(req.path.clone());
    //     let route_path: Vec<String> = clean_uri_to_vec(route.path.clone());

    //     if route.method.to_uppercase() != req.method.to_uppercase() {
    //         return (false, Values::new());
    //     }

    //     let (matches, parameters) = GroupRouter::parameters_route_match(route_path, request_path);

    //     if !matches {
    //         return (false, Values::new());
    //     }

    //     return (true, parameters);
    // }

    // fn parameters_route_match(route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
    //     let mut params: Values = Values::new();

    //     for (i, seg) in route_path.iter().enumerate() {
    //         if i > request_path.len() - 1 {
    //             return (false, Values::new());
    //         }

    //         let seg_match = request_path[i].clone();

    //         if seg == "*" {
    //             return (true, Values::new());
    //         }

    //         if seg == &seg_match {
    //             continue;
    //         }

    //         if PARAM_REGEX.is_match(&seg.to_string()) {
    //             params.insert(
    //                 seg.trim_start_matches('{').trim_end_matches('}').to_owned(),
    //                 seg_match
    //             );

    //             continue;
    //         }

    //         return (false, Values::new());
    //     }

    //     return (true, params)
    // }

    pub(crate) fn handle_middlewares(middlewares: &Middlewares, req: &'r mut Request, res: &'r mut Response, middlewares_ref: &MiddlewaresRef) -> Option<&'r mut Response> {
        for middleware_ref in  middlewares_ref {
            let middleware = middlewares.get(middleware_ref).unwrap();
            let mut next = Next::new();

            middleware(req, res, &mut next);

            if !next.is_move {
                return None;
            }
        }

        return Some(res);
    }
}

