use std::mem::take;

use crate::{
    request::Request,
    response::Response,
    router::{
        Group,
        Middleware,
        Route,
        RouteNotFoundCallback,
        Router,
        RouterNodes,
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
    pub(crate) not_found_callback: RouteNotFoundCallback,
}

pub(crate) struct ResolvedRoutes {
    pub web: WebRoutes,
    pub ws: WsRoutes,
    pub not_found: RouteNotFoundCallback,
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
        for mut node in take(&mut self.nodes) {
            let resolved = self.resolve_router_nodes(&mut node);

            self.web.extend(resolved.web);
            self.ws.extend(resolved.ws);

            if resolved.not_found.is_some() {
                self.not_found_callback = resolved.not_found;
            }
        }
    }

    pub async fn web_match<'g>(&mut self, req: &'g mut Request, res: &'g mut Response) -> Option<&'g mut Response> {
        for route in &mut self.web {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            // TODO: fix
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

    pub async fn ws_match<'g>(&'g mut self, req: &'g mut Request, res: &'g mut Response) -> Option<(&'g mut Route<Box<WsRoute>>, &'g mut Request, &'g mut Response)> {
        for route in &mut self.ws {
            let (is_match, parameters) = route.is_match(req);

            if !is_match {
                continue;
            }
            
            req.parameters = parameters;

            // TODO: fix
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

    fn resolve_router_nodes(&mut self, router: &mut Box<Router>) -> ResolvedRoutes {
        let mut resolved = ResolvedRoutes::new(
            take(&mut router.web),
            take(&mut router.ws), 
            take(&mut router.not_found)
        );

        if let Some(group) = router.group {
            resolved.resolve_group(router, group);
        }

        for node in &mut router.nodes {
            resolved.extend(&mut self.resolve_router_nodes(node));
        }

        return resolved;
    }
}

impl ResolvedRoutes {
    pub fn new(web_routes: WebRoutes, ws_routes: WsRoutes, not_found_callback: RouteNotFoundCallback) -> Self {
        return Self {
            web: web_routes,
            ws: ws_routes,
            not_found: not_found_callback,
        };
    }

    pub fn resolve_group(&mut self, router: &mut Router, group: Group) -> &mut Self {
        group(router);

        return self.append(
            take(&mut router.web),
            take(&mut router.ws),
            take(&mut router.not_found)
        );
    }

    pub fn extend(&mut self, resolved: &mut Self) -> &mut Self {
        return self.append(
            take(&mut resolved.web),
            take(&mut resolved.ws),
            take(&mut resolved.not_found)
        );
    }

    pub fn append(&mut self, web_routes: WebRoutes, ws_routes: WsRoutes, not_found: RouteNotFoundCallback) -> &mut Self {
        self.web.extend(web_routes);
        self.ws.extend(ws_routes);

        if not_found.is_some() {
            self.not_found = not_found;
        }

        return self;
    }
}