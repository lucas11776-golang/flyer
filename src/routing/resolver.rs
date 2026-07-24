
use crate::{routing::{router::Router, routes::Routes}, server::Server, utils::vec};

pub(crate) struct Resolver {}

impl Resolver {
    pub fn new(server: &mut Server) {
        Self::resolve(&mut server.routes, &mut server.routers);
        server.routers.clear();
    }

    fn resolve(server: &mut Routes, nodes: &mut Vec<Box<Router>>) {
        for router in nodes {
            Self::recursive(server, router);
        }
    }

    fn recursive(routes: &mut Routes, router: &mut Box<Router>) {
        for group in &router.groups {
            let mut middlewares = router.middlewares.clone();

            middlewares.extend(group.middlewares.clone());

            router.routers.push(Box::new(Router::new(
                router.server.clone(),
                group.subdomain.clone(),
                vec::merge(router.path.clone(), group.path.clone()),
                middlewares,
            )));

            let last = router.routers.len() - 1;

            group.call(router.routers[last].as_mut());
        }

        Self::resolve(routes, &mut router.routers);

        routes.http.append(&mut router.http);
        routes.websocket.append(&mut router.websocket);
    }
}