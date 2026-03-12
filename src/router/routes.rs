use crate::{request::Request, response::Response, router::{Route, WebRoute, WsRoute, middleware::call, next::Next}};

#[derive(Default)]
pub(crate) struct Routes {
    pub(crate) web: Vec<Route<WebRoute>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
    pub(crate) not_found_callback: Option<Route<WebRoute>>,
}

impl Routes {
    pub fn handle_web_request<'r>(&self, req: &'r mut Request, res: &'r mut Response) {
        for route in &self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if !self.handle_middlewares(req, res, &route.middlewares) {
                return;
            }

            (route.handler)(req, res);

            return;
        }

        if self.not_found_callback.is_some() {
            (self.not_found_callback.as_ref().unwrap().handler)(req, res);

            return;
        }

        res.status_code = 404;

        return;
    }

    pub fn handle_ws_request<'r>(&'r self, req: &'r mut Request, res: &'r mut Response) -> Option<&'r Route<WsRoute>> {
        for route in &self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if !self.handle_middlewares(req, res, &route.middlewares) {
                return None;
            }

            return Some(&route);
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
