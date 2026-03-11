use async_std::task::block_on;

use crate::{request::Request, response::Response, router::middleware::register, ws::Ws};

pub(crate) mod resolver;
pub(crate) mod middleware;
pub(crate) mod routes;

pub(crate) type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static;
pub(crate) type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static;
pub(crate) type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws) + Send + Sync + 'static;
pub(crate) type Group = for<'a> fn(&mut Router);

pub struct Route<Handler: ?Sized> {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) handler: Box<Handler>,
    pub(crate) middlewares: Vec<String>,
}

impl<'q, Handler: ?Sized> std::fmt::Debug for Route<Handler> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("middlewares", &self.middlewares)
            .finish()
    }
}

pub struct Router {
    pub(crate) web_routes: Vec<Box<Route<WebRoute>>>,
    pub(crate) ws_routes: Vec<Box<Route<WsRoute>>>,
    pub(crate) path: String,
    pub(crate) middlewares: Vec<String>,
    pub(crate) group: Option<Group>,
    pub(crate) routers: Vec<Box<Router>>,
}

pub fn join_url(url: Vec<String>) -> String {
    return url.iter()
        .map(|u| String::from(u.trim_matches('/'))).collect::<Vec<_>>()
        .join("/")
        .trim_matches('/')
        .to_string();
}

impl Router {
    pub fn get<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        self.route("GET", path, callback);
    }

    pub fn post<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        self.route("POST", path, callback);
    }

    pub fn route<C>(&mut self, method: &str, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static
    {
        self.web_routes.push(Box::new(Route {
            method: String::from(method.to_uppercase()),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: self.middlewares.clone()
        }));
    }
    
    pub fn group<'g>(&'g mut self, path: &str, group: Group) -> GroupRouter<'g> {
        let idx = self.routers.len();

        self.routers.push(Box::new(Router {
            web_routes: Vec::new(),
            ws_routes: Vec::new(),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            middlewares: self.middlewares.clone(),
            group: Some(group),
            routers: vec![],
        }));

        return GroupRouter::new(self.routers[idx].as_mut())
    }

    pub fn ws<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Ws) + Send + Sync + 'static
    {
        self.ws_routes.push(Box::new(Route {
            method: String::from("GET"),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, ws| block_on(callback(req, ws))),
            middlewares: vec![],
        }));
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
        }
    }

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        self.router.middleware(callback);
        return self;
    }
}


pub struct Next {
    pub(crate) is_move: bool,
}

impl Next {
    pub(crate) fn new() -> Self {
        return Self {
            is_move: false
        } 
    }

    pub fn handle<'a>(&mut self, res: &'a mut Response) -> &'a mut Response {
        self.is_move = true;

        return res;
    }
}
