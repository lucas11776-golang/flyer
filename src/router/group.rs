use std::{io::Result};
use regex::Regex;
use once_cell::sync::Lazy;

use futures::future::{Future, FutureExt};

use crate::{
    request::Request,
    response::{Response},
    router::{
        Middlewares, Route, WebRoute, WsRoute
    },
    utils::{
        url::clean_uri_to_vec,
        Values
    },
};

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex")
});

#[derive(Default)]
pub struct GroupRouter {
    pub(crate) web: Vec<Route<Box<WebRoute<'static>>>>,
    pub(crate) ws: Vec<Route<WsRoute>>,
    pub(crate) not_found_callback: Option<Box<WebRoute<'static>>>,
}

impl <'a>GroupRouter {
    pub fn new() -> Self {
        return GroupRouter {
            web: vec![],
            ws: vec![],
            not_found_callback: None,
        }
    }

    pub fn add_web_route<'s, C, F>(&mut self, method: &str, path: String, callback: C, middlewares: Middlewares)
    where
        C: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.web.push(Route{
            path: path,
            method: method.to_string(),
            route: Box::new(move |req: Request, res: Response| callback(req, res).boxed()),
            middlewares,
        });
    }

    pub async fn match_web_routes<'s>(& mut self, mut req: Request, mut res: Response) -> Result<Response>
    where
        'a: 's
     {
        for route in &mut self.web {
            let (matches, parameters) = GroupRouter::match_route(route, &mut req);

            if !matches {
                continue;
            }
            
            req.parameters = parameters;

            // if GroupRouter::handle_middlewares(req, res, &route.middlewares).is_none() {
            //     return Ok(res)
            // }

            return Ok((route.route)(req, res).await)
        }

        if self.not_found_callback.is_some() {
            return Ok(self.not_found_callback.as_ref() .unwrap()(req, res).await);
        }

        res.status_code = 404;

        return Ok(res)
    }

    // pub async fn match_ws_routes(&mut self, req: &'a mut Request, res: &'a mut Response) -> Option<&'a mut Ws> {
    //     for route in &mut self.ws {
    //         let (matches, parameters) = GroupRouter::match_route(route, req);

    //         if !matches {
    //             continue;
    //         }
            
    //         req.parameters = parameters;

    //         if GroupRouter::handle_middlewares(req, res, &route.middlewares).is_none() {
    //             return None;
    //         }

    //         let mut ws = res.ws.as_mut().unwrap();

    //         (route.route)(req, &mut ws);

    //         if ws.event.is_some() {
    //             ws.event.as_ref().unwrap()(Event::Ready()).await;
    //         }

    //         return Some(ws);
    //     }

    //     return None;
    // }

    fn match_route<T>(route: &mut Route<T>, req: &mut Request) -> (bool, Values) {
        let request_path: Vec<String> = clean_uri_to_vec(req.path.clone());
        let route_path: Vec<String> = clean_uri_to_vec(route.path.clone());

        if route.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        let (matches, parameters) = GroupRouter::parameters_route_match(route_path, request_path);

        if !matches {
            return (false, Values::new());
        }

        return (true, parameters);
    }

    fn parameters_route_match(route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
        let mut params: Values = Values::new();

        for (i, seg) in route_path.iter().enumerate() {
            if i > request_path.len() - 1 {
                return (false, Values::new());
            }

            let seg_match = request_path[i].clone();

            if seg == "*" {
                return (true, Values::new());
            }

            if seg == &seg_match {
                continue;
            }

            if PARAM_REGEX.is_match(&seg.to_string()) {
                params.insert(
                    seg.trim_start_matches('{').trim_end_matches('}').to_owned(),
                    seg_match
                );

                continue;
            }

            return (false, Values::new());
        }

        return (true, params)
    }

    // fn handle_middlewares(mut req:  Request, mut res: Response, middlewares: &Middlewares) -> Option<Response> {
    //     // for middleware in  middlewares {
    //     //     let mut move_to_next: bool = false;

    //     //     let next: Next = Next{
    //     //         is_next: &mut move_to_next,
    //     //         response: &mut new_response(None),
    //     //     };

    //     //     middleware(req, res, next);

    //     //     if !move_to_next {
    //     //         return None;
    //     //     }
    //     // }

    //     return Some(res);
    // }
}

