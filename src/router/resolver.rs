use std::mem::take;

use crate::{router::{Route, Router, WebRoute, WsRoute}, server::Server};


#[derive(Default)]
struct ResolvedRoutes {
    pub(crate) web: Vec<Route<WebRoute>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
}

impl ResolvedRoutes {
    pub(crate) fn new(routers: &mut Vec<Box<Router>>) -> Self {
        let mut resolved = ResolvedRoutes::default();

        for router in &mut *routers {
            resolved = Self::resolve(resolved, router)
        }

        return resolved;
    }
    
    pub(crate) fn resolve<'q>(mut resolved: ResolvedRoutes, router: &mut Router) -> ResolvedRoutes {
        for node in &mut *router.routers {
            if let Some(group) = node.group {
                group(node);
            }

            resolved.web.extend(take(&mut node.web));
            resolved.ws.extend(take(&mut node.ws));

            resolved = Self::resolve(resolved, node);

            if node.route_not_found_callback.is_some() {
                resolved.not_found_callback = take(&mut node.route_not_found_callback);
            }
        }

        resolved.web.extend(take(&mut router.web));
        resolved.ws.extend(take(&mut router.ws));

        if router.route_not_found_callback.is_some() {
            resolved.not_found_callback = take(&mut router.route_not_found_callback);
        }

        return resolved;
    }
}


pub(crate) fn resolve(server: &mut Server)  {
    let mut resolved = ResolvedRoutes::new(&mut server.routers);

    server.routes.web.extend(take(&mut resolved.web));
    server.routes.ws.extend(take(&mut resolved.ws));

    if resolved.not_found_callback.is_some() {
        server.routes.not_found_callback = take(&mut resolved.not_found_callback);
    }

    server.routers.clear();
}