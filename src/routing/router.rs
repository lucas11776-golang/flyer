use std::{collections::HashSet, future::Future};

use crate::{
    request::Request,
    response::Response,
    routing::{Group, HttpHandler, WebsocketHandler, next::Next, route::Route},
    server::Server,
    utils::{mem::Instance, route::middleware_resolver,
        url::{self, clean},
        vec,
    }, websocket::Websocket,
};

pub struct Router {
    pub(crate) server: Instance<Server>,
    pub(crate) subdomain: String,
    pub(crate) path: Vec<String>,
    pub(crate) http: Vec<Route<HttpHandler>>,
    pub(crate) websocket: Vec<Route<WebsocketHandler>>,
    pub(crate) routers: Vec<Router>,
    pub(crate) groups: Vec<GroupRouter>,
    pub(crate) middlewares: HashSet<String>,
    pub(crate) not_found_callback: Option<HttpHandler>,
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Self {
            server: self.server.clone(),
            subdomain: self.subdomain.clone(),
            path: self.path.clone(),
            http: Vec::new(),
            websocket: Vec::new(),
            routers: Vec::new(),
            groups: Vec::new(),
            middlewares: self.middlewares.clone(),
            not_found_callback: None,
        }
    }
}

impl From<&Router> for Router {
    fn from(value: &Router) -> Self {
        value.clone()
    }
}

macro_rules! impl_http_method {
    ($fn_name:ident, $method:expr) => {
        pub fn $fn_name<C, Fut>(&mut self, path: impl Into<String>, callback: C) -> &mut Route<HttpHandler>
        where
            C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + 'static,
        {
            self.route($method, path, callback)
        }
    };
}

impl Router {
    pub(crate) fn new(
        server: Instance<Server>,
        subdomain: String,
        path: Vec<String>,
        middlewares: HashSet<String>,
    ) -> Self {
        Self {
            server,
            subdomain,
            path,
            http: Vec::new(),
            websocket: Vec::new(),
            routers: Vec::new(),
            groups: Vec::new(),
            middlewares: middlewares,
            not_found_callback: None,
        }
    }

    pub(crate) fn fresh(ptr: Instance<Server>) -> Self {
        Self::new(ptr, String::new(), Vec::new(), HashSet::new())
    }

    impl_http_method!(get, "GET");
    impl_http_method!(post, "POST");
    impl_http_method!(put, "PUT");
    impl_http_method!(patch, "PATCH");
    impl_http_method!(delete, "DELETE");
    impl_http_method!(copy, "COPY");
    impl_http_method!(head, "HEAD");
    impl_http_method!(options, "OPTION");

    pub fn route<C, Fut>(&mut self, method: impl Into<String>, path: impl Into<String>, callback: C) -> &mut Route<HttpHandler>
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.http.push(Route {
            server: self.server.clone(),
            method: method.into().to_uppercase(),
            subdomain: self.subdomain.clone(),
            path: vec::merge(self.path.clone(), url::clean(path)),
            handler: Box::new(move |req, res| Box::pin(callback(req, res))),
            middlewares: self.middlewares.clone(),
        });

        self.http.last_mut().unwrap()
    }

    pub fn not_found<C, Fut>(&mut self, callback: C)
    where
        C: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.not_found_callback = Some(Box::new(move |req, res| Box::pin(callback(req, res))));
    }

    pub fn ws<C, Fut>(&mut self, path: impl Into<String>, callback: C) -> &mut Route<WebsocketHandler>
    where
        C: Fn(Request, Websocket) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Websocket> + Send + 'static,
    {
        self.websocket.push(Route {
            server: self.server.clone(),
            method: "GET".to_string(),
            subdomain: self.subdomain.clone(),
            path: vec::merge(self.path.clone(), url::clean(path)),
            handler: Box::new(move |req, ws| Box::pin(callback(req, ws))),
            middlewares: self.middlewares.clone(),
        });

        self.websocket.last_mut().unwrap()
    }

    pub fn group(&mut self, path: &str, group: Group) -> &mut GroupRouter {
        self.groups.push(GroupRouter::new(
            self.server.clone(),
            self.subdomain.clone(),
            clean(path),
            group,
        ));
        self.groups.last_mut().unwrap()
    }

    pub fn subdomain(&mut self, subdomain: impl Into<String>, group: Group) -> &mut GroupRouter {
        self.groups.push(GroupRouter::new(
            self.server.clone(),
            subdomain.into(),
            self.path.clone(),
            group,
        ));
        self.groups.last_mut().unwrap()
    }

    pub fn middleware<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        middleware_resolver(
            self.server.clone(),
            Box::new(move |req, res, next| Box::pin(callback(req, res, next))),
            &mut self.middlewares,
        );

        self
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
    pub(crate) fn new(
        server: Instance<Server>,
        subdomain: String,
        path: Vec<String>,
        group: Group,
    ) -> Self {
        Self {
            server,
            path,
            handler: group,
            middlewares: HashSet::new(),
            subdomain,
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
            &mut self.middlewares,
        );

        self
    }

    pub fn call(&self, router: &mut Router) {
        (self.handler)(router);
    }
}