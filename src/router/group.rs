use std::mem::transmute_copy;

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

impl <'r>GroupRouter {
    pub fn new() -> Self {
        return GroupRouter {
            web: vec![],
            ws: vec![],
            not_found_callback: None,
            nodes: vec![]
        }
    }
    
    pub fn resolve_nodes(&mut self) {
        for mut node in &mut self.nodes {
            let (web, ws, not_found) = GroupRouter::resolve_router_nodes(&mut node);

            self.web.extend(web);
            self.ws.extend(ws);

            if not_found.is_some() {
                self.not_found_callback = not_found;
            }
        }
    }

    pub async fn match_web_routes(&mut self, req: &'r mut Request, res: &'r mut Response) -> Option<&'r mut Response> {
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

    pub async fn match_ws_routes<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Option<(&'a mut Route<Box<WsRoute>>, &'a mut Request, &'a mut Response)> {
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

    fn handle_middlewares(middlewares: &Vec<Box<Middleware>>, req: &'r mut Request, res: &'r mut Response) -> Option<&'r mut Response> {
        for middleware in  middlewares {
            let mut next = Next::new();

            middleware(req, res, &mut next);

            if !next.is_move {
                return None;
            }
        }

        return Some(res);
    }

    fn resolve_router_nodes(router: &mut Box<Router>) -> (Vec<Route<Box<WebRoute>>>, Vec<Route<Box<WsRoute>>>, Option<Box<WebRoute>>) {
        let mut web: Vec<Route<Box<WebRoute>>> = vec![];
        let mut ws: Vec<Route<Box<WsRoute>>> = vec![];
        let mut not_found: Option<Box<WebRoute>> = None;

        if router.group .is_some() {
            router.group.as_mut().unwrap()(router);

            web.extend::<Vec<Route<Box<WebRoute>>>>(unsafe { transmute_copy(&mut router.web) });
            ws.extend::<Vec<Route<Box<WsRoute>>>>(unsafe { transmute_copy(&mut router.ws) });
        }

        if router.not_found_callback.is_some() {
            not_found = unsafe { transmute_copy(&mut router.not_found_callback) };
        }
        
        for node in &mut router.router_nodes {
            let (_web, _ws, _not_found) = GroupRouter::resolve_router_nodes(node);

            web.extend(_web);
            ws.extend(_ws);
        }

        return (web, ws, not_found);
    }
}

