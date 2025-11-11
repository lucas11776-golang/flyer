use std::collections::HashMap;

use futures::executor::block_on;

use crate::{
    request::Request,
    response::Response,
    router::{Middlewares, MiddlewaresRef, Next, Route, WebRoute, WsRoute}
};

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

    pub async fn match_web_routes(&mut self, req: &'r mut Request, res: &'r mut Response) -> Option<&'r mut Response> {
        for route in &mut self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if Self::handle_middlewares(&self.middlewares, req, res, &route.middlewares).is_none() {
                return Some(res)
            }

            return Some((route.route)(req, res))
        }

        if self.not_found_callback.is_some() {
            return Some(self.not_found_callback.as_ref() .unwrap()(req, res));
        }

        res.status_code = 404;

        return None;
    }

    pub async fn match_ws_routes<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Option<(&'a mut Route<Box<WsRoute>>, &'a mut Request, &'a mut Response)> {
        for route in &mut self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if Self::handle_middlewares(&self.middlewares, req, res, &route.middlewares).is_none() {
                return None;
            }

            return Some((route, req, res));
        }

        return None;
    }

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

