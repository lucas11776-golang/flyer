use async_std::task::block_on;

use crate::{
    request::Request,
    response::Response,
    router::{middleware::register, next::Next, route::Route},
    utils::url::join_url,
    ws::Ws
};

pub(crate) mod resolver;
pub(crate) mod middleware;
pub(crate) mod routes;
pub mod route;
pub mod next;

pub(crate) type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static;
pub(crate) type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static;
pub(crate) type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws) + Send + Sync + 'static;
pub(crate) type Group = for<'a> fn(&mut Router);

pub struct Router {
    pub(crate) web: Vec<Route<WebRoute>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
    pub(crate) path: String,
    pub(crate) middlewares: Vec<String>,
    pub(crate) group: Option<Group>,
    pub(crate) routers: Vec<Box<Router>>,
    pub(crate) route_not_found_callback: Option<Box<WebRoute>>,
}

impl Router {
    pub fn get<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("GET", path, callback);
    }

    pub fn post<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("POST", path, callback);
    }

    pub fn put<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("PUT", path, callback);
    }

    pub fn patch<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("PATCH", path, callback);
    }

    pub fn delete<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("DELETE", path, callback);
    }

    pub fn head<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("HEAD", path, callback);
    }

    pub fn options<C>(&mut self, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        return self.route("OPTIONS", path, callback);
    }

    pub fn route<C>(&mut self, method: &str, path: &str, callback: C) -> &mut Route<WebRoute>
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static
    {
        let idx = self.web.len();

        self.web.push(Route {
            subdomain: String::new(),
            method: String::from(method.to_uppercase()),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: self.middlewares.clone()
        });

        return &mut self.web[idx];
    }
    
    pub fn group<'g>(&'g mut self, path: &str, group: Group) -> GroupRouter<'g> {
        let idx = self.routers.len();
        
        self.routers.push(Box::new(Router {
            web: Vec::new(),
            ws: Vec::new(),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            middlewares: self.middlewares.clone(),
            group: Some(group),
            routers: vec![],
            route_not_found_callback: None
        }));

        return GroupRouter::new(self.routers[idx].as_mut());
    }

    pub fn ws<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Ws) + Send + Sync + 'static
    {
        self.ws.push(Box::new(Route {
            subdomain: String::new(),
            method: String::from("GET"),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, ws| block_on(callback(req, ws))),
            middlewares: vec![],
        }));
    }

    pub fn not_found<C>(&mut self, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        self.route_not_found_callback = Some(Box::new(move |req, res| block_on(callback(req, res))));
    }

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        self.middlewares
            .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));

        return self;
    }

}

pub struct GroupRouter<'g> {
    router: &'g mut Router
}

impl <'g>GroupRouter<'g> {
    pub(crate) fn new(router: &'g mut Router) -> Self {
        return Self {
            router: router
        };
    }

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        self.router.middleware(callback);
        return self;
    }
}
