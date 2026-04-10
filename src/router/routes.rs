use std::fmt;
use crate::{
    request::Request,
    response::Response,
    router::{Route, WebRoute, WsRoute, middleware::call, next::Next}
};

#[derive(Default)]
pub(crate) struct Routes {
    pub(crate) web: Vec<Route<WebRoute>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
}

impl fmt::Debug for Routes {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Routes")
            .field("web", &self.web)
            .field("ws", &self.ws)
            .field("not_found_callback", &self.not_found_callback.as_ref().map(|_| "Fn(...)"))
            .finish()
    }
}

impl Routes {
    pub fn handle_web_request<'r>(&self, req: &'r mut Request, res: &'r mut Response) {
        for route in &self.web {
            let (is_match, parameters) = route.is_match(req);
            
            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            return match self.handle_middlewares(req, res, &route.middlewares) {
                true => { (route.handler)(req, res); },
                false => {},
            };
        }

        return match &self.not_found_callback {
            Some(callback) => { callback(req, res); },
            None => { res.status_code = 404; }
        };
    }

    pub fn handle_ws_request<'r>(&'r self, req: &'r mut Request, res: &'r mut Response) -> Option<(&'r Route<WsRoute>, &'r mut Request, &'r mut Response)> {
        for route in &self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }

            req.parameters = parameters;

            return match self.handle_middlewares(req, res, &route.middlewares) {
                true => Some((&route, req, res)),
                false => None,
            };
        }

        return None;
    }

    fn handle_middlewares<'g>(&self, req: &'g mut Request, res: &'g mut Response, middlewares: &Vec<String>) -> bool{
        for pointer in  middlewares {
            let mut next = Next::new();

            call(String::from(pointer), req, res, &mut next);

            if !next.is_move {
                return false;
            }
        }

        return true;
    }
}
