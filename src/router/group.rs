use std::mem::{take, transmute_copy};

use crate::{
    request::Request,
    response::Response,
    router::{
        Middleware,
        Route,
        Router,
        RouterNodes,
        WebRoute,
        WebRoutes,
        WsRoute,
        WsRoutes,
        next::Next
    }
};

pub struct GroupRouter {
    pub(crate) web: WebRoutes,
    pub(crate) ws: WsRoutes,
    pub(crate) nodes: RouterNodes,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
}

impl GroupRouter {
    pub fn new() -> Self {
        return GroupRouter {
            web: vec![],
            ws: vec![],
            not_found_callback: None,
            nodes: vec![]
        }
    }
    
    pub fn init(&mut self) {


        for mut node in &mut self.nodes {
            let (web, ws, not_found) = GroupRouter::resolve_router_nodes(&mut node);




            self.web.extend(web);
            self.ws.extend(ws);



            if not_found.is_some() {
                self.not_found_callback = not_found;
            }
        }
    }

    pub async fn match_web_routes<'g>(&mut self, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
        for route in &mut self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if Self::handle_middlewares(&route.middlewares, req, res).is_none() {
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

    pub async fn match_ws_routes<'g>(&'g mut self, req: &'g mut Request, res: &'g mut Response) -> Option<(&'g mut Route<Box<WsRoute>>, &'g mut Request, &'g mut Response)> {
        for route in &mut self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            if Self::handle_middlewares(&route.middlewares, req, res).is_none() {
                return None;
            }
            return Some((route, req, res));
        }

        return None;
    }

    fn handle_middlewares<'g>(middlewares: &Vec<Box<Middleware>>, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
        for middleware in  middlewares {
            let mut next = Next::new();

            middleware(req, res, &mut next);

            if !next.is_move {
                return None;
            }
        }

        return Some(res);
    }

    // TODO: working ref...
    fn resolve_router_nodes(router: &mut Box<Router>) -> (Vec<Route<Box<WebRoute>>>, Vec<Route<Box<WsRoute>>>, Option<Box<WebRoute>>) {
        let mut web: Vec<Route<Box<WebRoute>>> = take(&mut router.web);
        let mut ws: Vec<Route<Box<WsRoute>>> = take(&mut router.ws);
        let mut not_found: Option<Box<WebRoute>> = take(&mut router.not_found_callback);

        if let Some(group) = router.group {
            group(router);

            web.extend(take(&mut router.web));
            ws.extend(take(&mut router.ws));

            if router.not_found_callback.is_some() {
                not_found = take(&mut router.not_found_callback);
            }
        }

        if router.not_found_callback.is_some() {
            not_found = take(&mut router.not_found_callback);
        }
        
        for node in &mut router.router_nodes {
            let (temp_web, temp_ws, temp_not_found) = GroupRouter::resolve_router_nodes(node);

            web.extend(temp_web);
            ws.extend(temp_ws);

            if temp_not_found.is_some() {
                not_found = temp_not_found;
            }
        }

        return (web, ws, not_found);
    }
}

