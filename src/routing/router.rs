use std::collections::HashSet;

use crate::{
    request::Request, response::Response, routing::{Group, HttpHandler,WebsocketHandler, next::Next, route::Route}, server::Server, utils::{mem::Instance, route::middleware_resolver, url::{self, clean}, vec}, websocket::Websocket
};

pub struct Router {
    pub(crate) server: Instance<Server>,
    pub(crate) subdomain: String,
    pub(crate) path: Vec<String>,
    pub(crate) http: Vec<Box<Route<HttpHandler>>>,
    pub(crate) websocket: Vec<Box<Route<WebsocketHandler>>>,
    pub(crate) routers: Vec<Box<Router>>,
    pub(crate) groups: Vec<Box<GroupRouter>>,
    pub(crate) middlewares: HashSet<String>,
}

impl From<&mut Box<Router>> for Router {
    fn from(value: &mut Box<Router>) -> Self {
        return Self {
            server: value.server.clone(),
            subdomain: value.subdomain.clone(),
            path: value.path.clone(),
            http: Vec::new(),
            websocket: Vec::new(),
            routers: Vec::new(),
            groups: Vec::new(),
            middlewares: value.middlewares.clone(),
        }
    }
}

impl Clone for Router {
    fn clone(&self) -> Self {
        return Self {
            server: self.server.clone(),
            subdomain: self.subdomain.clone(),
            path: self.path.clone(),
            http: Default::default(),
            websocket: Default::default(),
            routers: Default::default(),
            groups: Default::default(),
            middlewares: self.middlewares.clone()
        };
    }
}

impl Router {
    pub(crate) fn new(ptr: Instance<Server>, subdomain: String, path: Vec<String>, middlewares: HashSet<String>) -> Self {
        return Self {
            server: ptr,
            subdomain: subdomain,
            path: path,
            http: Vec::new(),
            websocket: Vec::new(),
            routers: Vec::new(),
            groups: Vec::new(),
            middlewares,
        };
    }

    pub(crate) fn fresh(ptr: Instance<Server>) -> Self {
        return Self::new(ptr, String::new(), Vec::new(), HashSet::new());
    }

    pub fn get<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("GET", path, callback);
    }

    pub fn post<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("POST", path, callback);
    }

    pub fn put<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("PUT", path, callback);
    }

    pub fn patch<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("PATCH", path, callback);
    }

    pub fn delete<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("DELETE", path, callback);
    }

    pub fn copy<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("COPY", path, callback);
    }

    pub fn head<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("HEAD", path, callback);
    }

    pub fn options<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        return self.route("OPTION", path, callback);
    }

    pub fn route<C, Fut>(&mut self, method: &str, path: &str, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let server = self.server.clone();

        self.http.push(Box::new(Route {
            server: server,
            method: String::from(method.to_uppercase()),
            subdomain: self.subdomain.clone(),
            path: vec::merge(self.path.clone(), url::clean(path)),
            handler: Box::new(move |req, res| Box::pin(callback(req, res))),
            middlewares: self.middlewares.clone(),
        }));

        let last = self.http.len() - 1;

        return self.http[last].as_mut();
    }

    pub fn ws<C, Fut>(&mut self, path: &str, callback: C) -> &mut Route<WebsocketHandler>
    where
        C: Fn(Request, Websocket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let server = self.server.clone();
        
        self.websocket.push(Box::new(Route {
            server: server,
            method: String::from("GET"),
            subdomain: self.subdomain.clone(),
            path: vec::merge(self.path.clone(), url::clean(path)),
            handler: Box::new(move |req, ws| Box::pin(callback(req, ws))),
            middlewares: self.middlewares.clone(),
        }));

        return self.websocket.last_mut().unwrap();
    }

    pub fn group(&mut self, path: &str, group: Group) -> &mut GroupRouter {
        self
            .groups
            .push(Box::new(GroupRouter::new(self.server.clone(), self.subdomain.clone(), clean(path), group)));
        return self
            .groups
            .last_mut()
            .unwrap();
    }

    pub fn subdomain(&mut self, subdomain: impl Into<String>, group: Group) -> &mut GroupRouter {
        self
            .groups
            .push(Box::new(GroupRouter::new(self.server.clone(), subdomain.into(), self.path.clone(), group)));
        return self
            .groups
            .last_mut()
            .unwrap();
    }

    pub fn middleware<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        middleware_resolver(
            self.server.clone(),
            Box::new(move |req, res, next| Box::pin(callback(req, res, next))),
            &mut self.middlewares
        );

        return self;
    }

}

pub struct GroupRouter {
    pub(crate) server: Instance<Server>,
    pub(crate) path: Vec<String>,
    pub(crate) handler: Group,
    pub(crate) middlewares: HashSet<String>,
    pub(crate) subdomain: String,
}

impl GroupRouter {
    pub(crate) fn new(ptr: Instance<Server>, subdomain: String, path: Vec<String>, group: Group) -> Self {
        return Self {
            server: ptr,
            path: path,
            handler: group,
            middlewares: HashSet::new(),
            subdomain: subdomain,
        }
    }

    pub fn middleware<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        middleware_resolver(
            self.server.clone(),
            Box::new(move |req, res, next| Box::pin(callback(req, res, next))),
            &mut self.middlewares
        );

        return self;
    }

    pub fn call(&self, router: &mut Router) {
        (self.handler)(router);
    }
}