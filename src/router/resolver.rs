use std::mem::take;

use crate::{router::{Route, Router, WebRoute, WsRoute}, server::Server};


#[derive(Default)]
struct ResolvedRoutes {
    pub(crate) web: Vec<Route<WebRoute>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>
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
        }


        resolved.web.extend(take(&mut router.web));
        resolved.ws.extend(take(&mut router.ws));

        return resolved;
    }
}


pub(crate) fn resolve(server: &mut Server)  {
    let mut resolved = ResolvedRoutes::new(&mut server.routers);

    server.routes.web.extend(take(&mut resolved.web));
    server.routes.ws.extend(take(&mut resolved.ws));

    server.routers.clear();
}