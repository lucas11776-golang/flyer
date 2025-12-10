
use crate::{
    request::Request,
    response::Response,
    router::{
        MiddlewaresPointers,
        Route,
        RouteNotFoundCallback,
        RouterNodes,
        WebRoutes,
        WsRoute,
        WsRoutes,
        middleware::call,
        next::Next,
        resolver::resolve_router_nodes
    }
};

pub struct GroupRouter {
    pub(crate) web: WebRoutes,
    pub(crate) ws: WsRoutes,
    pub(crate) nodes: RouterNodes,
    pub(crate) not_found_callback: RouteNotFoundCallback,
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
        let (web, ws, not_found) = resolve_router_nodes(&mut self.nodes);

        self.web.extend(web);
        self.ws.extend(ws);

        self.not_found_callback = not_found;
    }

    pub async fn web_match<'g>(&mut self, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
        for route in &mut self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            // TODO: fix
            if Self::handle_middlewares(route.middlewares.clone(), req, res).is_none() {
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

            return Some((route, req, res));
        }

        return None;
    }

    fn handle_middlewares<'g>(middlewares: MiddlewaresPointers, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
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
