pub mod group;

use std::collections::HashMap;
use std::mem;
use regex::Regex;
use once_cell::sync::Lazy;

use futures::executor::block_on;

use crate::utils::{Values, merge};
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::{clean_url, clean_uri_to_vec};

pub type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync;
pub type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws) -> () + Send + Sync;
pub type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static;
pub type Middlewares = HashMap<String, Box<Middleware>>;
pub type MiddlewaresRef = Vec<String>;
pub type Group = for<'a> fn(&'a mut Router);

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex")
});

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Vec<Box<Middleware>>,
}

pub struct Next {
    pub(crate) is_move: bool,
}

pub struct Router {
    pub(crate) web: Vec<Route<Box<WebRoute>>>,
    pub(crate) ws: Vec<Route<Box<WsRoute>>>,
    pub(crate) path: Vec<String>,
    pub(crate) middleware: Vec<Box<Middleware>>,
    pub(crate) group: Option<Group>,
    pub(crate) nodes: Vec<Box<Router>>,
    pub(crate) not_found_callback: Option<Box<WebRoute>>,
}

impl <'r, R> Route<R> {
    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.middlewares.push(Box::new(move |req, res, next| block_on(callback(req, res, next))));

        return self;
    }

    pub(crate) fn is_match(&mut self, req: &'r mut Request) -> (bool, Values) {
        let request_path: Vec<String> = clean_uri_to_vec(req.path.clone());
        let route_path: Vec<String> = clean_uri_to_vec(self.path.clone());

        if self.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        return self.parameters_route_match(route_path, request_path);
    }
    
    fn parameters_route_match(&mut self, route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
        let mut params: Values = Values::new();

        for (i, _) in request_path.iter().enumerate() {
            if i > route_path.len() - 1 {
                return (false, Values::new());
            }

            if route_path[i] == "*" {
                return (true, params);
            }

            if route_path[i] == request_path[i] {
                // Off guard
                if request_path.len() - 1 == i && route_path.len() > request_path.len() {
                    return (false, Values::new());
                }

                continue;
            }

            if PARAM_REGEX.is_match(&route_path[i].to_string()) {
                params.insert(route_path[i].trim_start_matches('{').trim_end_matches('}').to_owned(), request_path[i].to_string());

                continue;
            }

            // Off guard
            if request_path.len() - 1 == i && route_path.len() > request_path.len() {
                return (false, Values::new());
            }

            return (false, Values::new());
        }

        return (true, params)
    }
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

pub struct GroupRouting<'r> {
    pub(crate) router: &'r mut Box<Router>
}

impl <'r>GroupRouting<'r> {
    pub(crate) fn new(router: &'r mut Box<Router>) -> GroupRouting<'r> {
        return Self {
            router: router,
        }
    }

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.router.middleware.push(Box::new(move |req, res, next| block_on(callback(req, res, next))));

        return self;
    }
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
        let path = self.path(path).join("/");
        let middlewares: Vec<Box<Middleware>> = unsafe { mem::transmute_copy(&mut self.middleware) };

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
        let path = self.path(path).join("/");
        let middlewares: Vec<Box<Middleware>> = unsafe { mem::transmute_copy(&mut self.middleware) };

        self.ws.push(Route{
            path: path,
            method: "GET".to_string(),
            route: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: middlewares,
        });

        return &mut self.ws[idx];
    }

    pub fn group<'g>(&'g mut self , path: &str, group: Group) -> GroupRouting<'g> {
        let idx = self.nodes.len();
        let path = self.path(path);
        let middlewares: Vec<Box<Middleware>> = unsafe { mem::transmute_copy(&mut self.middleware) };

        self.nodes.push(Box::new(Router{
            web: vec![],
            ws: vec![],
            path: path,
            middleware: middlewares,
            group: Some(group),
            nodes: vec![],
            not_found_callback: None,
        }));

        return GroupRouting::new(&mut self.nodes[idx])
    }

    fn path(&mut self, path: &str) -> Vec<String> {
        return merge(vec![self.path.clone(), vec![path.to_string()]]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }
}