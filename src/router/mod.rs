pub mod group;
pub mod route;
pub mod next;

use std::mem;

use futures::executor::block_on;

use crate::router::next::Next;
use crate::router::route::{GroupRoute, Route};
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::join_paths;

pub type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync;

pub type WebRoutes = Vec<Route<Box<WebRoute>>>;

pub type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws) + Send + Sync;

pub type WsRoutes = Vec<Route<Box<WsRoute>>>;

pub type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync;

pub type Middlewares = Vec<Box<Middleware>>;

pub type Group = for<'a> fn(&'a mut Router);

pub type RouterNodes = Vec<Box<Router>>;

pub struct Router {
    pub(crate) web: WebRoutes,
    pub(crate) ws: WsRoutes,
    pub(crate) path: Vec<String>,
    pub(crate) middlewares: Middlewares,
    pub(crate) group: Option<Group>,
    pub(crate) router_nodes: RouterNodes,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
}

impl <'r>Router {
    pub fn get<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        return self.route("GET", path, callback);
    }

    pub fn post<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        
    {
        return self.route("POST", path, callback);
    }

    pub fn patch<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        return self.route("PATCH", path, callback);
    }

    pub fn put<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        return self.route("PUT", path, callback);
    }

    pub fn delete<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        return self.route("DELETE", path, callback);
    }   

    pub fn options<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        return self.route("OPTIONS", path, callback);
    }

    pub fn head<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("CONNECT", path, callback);
    }

    pub fn route<C>(&mut self, method: &str, path: &str, callback: C) -> &mut Route<Box<WebRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        let idx = self.web.len();
        let path = join_paths(self.path.join("/"), path.to_string()).join("/");
        let middlewares: Vec<Box<Middleware>> = unsafe { mem::transmute_copy(&mut self.middlewares) };

        self.web.push(Route{
            path: path,
            method: method.to_string(),
            route: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: middlewares,
        });

        return &mut self.web[idx];
    }

    pub fn not_found<C>(&mut self, callback: C)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.not_found_callback = Some(Box::new(move |req, res| block_on(callback(req, res))));
    }

    pub fn ws<C>(&mut self, path: &str, callback: C) -> &mut Route<Box<WsRoute>>
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Ws), Output = ()> + Send + Sync + 'static,
    {
        let idx = self.web.len();
        let path = join_paths(self.path.join("/"), path.to_string()).join("/");
        let middlewares: Middlewares = unsafe { mem::transmute_copy(&mut self.middlewares) };

        self.ws.push(Route{
            path: path,
            method: "GET".to_string(),
            route: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: middlewares,
        });

        return &mut self.ws[idx];
    }

    pub fn group<'g>(&'g mut self , path: &str, group: Group) -> GroupRoute<'g> {
        let idx = self.router_nodes.len();
        let path = join_paths(self.path.join("/"), path.to_string());
        let middlewares: Middlewares = unsafe { mem::transmute_copy(&mut self.middlewares) };

        self.router_nodes.push(Box::new(Router{
            web: vec![],
            ws: vec![],
            path: path,
            middlewares,
            group: Some(group),
            router_nodes: vec![],
            not_found_callback: None,
        }));

        return GroupRoute::new(&mut self.router_nodes[idx])
    }
}
