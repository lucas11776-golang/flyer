use anyhow::Result;

use crate::{request::Request, response::Response, router::{Route, WebRoute, WsRoute, middleware::call, next::Next}};

#[derive(Debug, Default)]
pub(crate) struct Routes {
    pub(crate) web: Vec<Box<Route<WebRoute>>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
}

impl Routes {
    // pub(crate) async fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
    //     println!("HANDLING REQUESt");
    //     return Ok((req, res));
    // }
}






impl Routes {
    pub async fn web_match<'g>(&self, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
        for route in &self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            // TODO: fix
            if Self::handle_middlewares(route.middlewares.clone(), req, res).is_none() {
                return Some(res)
            }

            return Some((route.handler)(req, res))
        }

        // if self.not_found_callback.is_some() {
        //     return Some(self.not_found_callback.as_ref() .unwrap()(req, res));
        // }

        res.status_code = 404;

        return None;
    }

    pub async fn ws_match<'g>(&'g mut self, req: &'g mut Request, res: &'g mut Response) -> Option<(&'g mut Route<Box<WsRoute>>, &'g mut Request, &'g mut Response)> {
        for route in &mut self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            // TODO: fix
            if Self::handle_middlewares(route.middlewares.clone(), req, res).is_none() {
                return None;
            }

            // return Some((route, req, res));
        }

        return None;
    }

    fn handle_middlewares<'g>(middlewares: Vec<String>, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
        for pointer in  middlewares {
            let mut next = Next::new();

            call(pointer, req, res, &mut next);

            if !next.is_move {
                return None;
            }
        }

        return Some(res);
    }
}
