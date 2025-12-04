use std::mem::take;

use crate::router::{Group, RouteNotFoundCallback, Router, RouterNodes, WebRoutes, WsRoutes};




pub(crate) struct Resolver {
    pub nodes: RouterNodes,
    pub web: WebRoutes,
    pub ws: WsRoutes,
    pub not_found: RouteNotFoundCallback,
}

impl <'r>Resolver {
    pub fn new(nodes: RouterNodes) -> Self {
        return Self {
            nodes: nodes,
            web: WebRoutes::new(),
            ws: WsRoutes::new(),
            not_found: None,
        };
    }

    pub fn resolve(&mut self) -> &mut Self {
        for node in &mut self.nodes {

        }


        return self;
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