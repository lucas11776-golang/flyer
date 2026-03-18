use crate::{router::{Route, Router, WebRoute, WsRoute}, server::Server};

#[derive(Default)]
pub(crate) struct RouterResolver {
    pub(crate) web: Vec<Route<WebRoute>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
}

impl RouterResolver {
    pub fn resolve(server: &mut Server) {
        let resolved = Self::new(&mut server.routers);

        server.routes.web.extend(resolved.web);
        server.routes.ws.extend(resolved.ws);

        if let Some(cb) = resolved.not_found_callback {
            server.routes.not_found_callback = Some(cb);
        }

        server.routers.clear();
    }

    fn new(routers: &mut Vec<Box<Router>>) -> Self {
        let mut resolved = Self::default();

        for router in routers {
            resolved.resolve_recursive(router);
        }

        resolved
    }

    fn resolve_recursive(&mut self, router: &mut Router) {
        for node in &mut *router.routers {
            if let Some(group) = node.group {
                group(node);
            }

            self.resolve_recursive(node);
        }

        self.web.append(&mut router.web);
        self.ws.append(&mut router.ws);

        if let Some(callback) = router.route_not_found_callback.take() {
            self.not_found_callback = Some(callback);
        }
    }
}