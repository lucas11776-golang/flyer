pub mod group;

use std::collections::HashMap;

use futures::executor::block_on;

use crate::{router::group::GroupRouter};
use crate::utils::{Values, merge};
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::{clean_url, clean_uri_to_vec};

pub type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync;
pub type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws) -> () + Send + Sync;
pub type Middleware = for<'a> fn (&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response;
pub type Middlewares = HashMap<String, Box<Middleware>>;
pub type MiddlewaresRef = Vec<String>;
pub type Group = fn(Router);

pub struct Next {
    pub(crate) is_move: bool,
}

pub struct Router<'r> {
    pub(crate) router: &'r mut GroupRouter,
    pub(crate) path: Vec<String>,
    pub(crate) middleware: MiddlewaresRef,
}

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: MiddlewaresRef,
}


use regex::Regex;
use once_cell::sync::Lazy;

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex")
});


impl <'r, R> Route<R> {
    pub fn is_match(&mut self, req: &'r mut Request) -> (bool, Values) {
        let request_path: Vec<String> = clean_uri_to_vec(req.path.clone());
        let route_path: Vec<String> = clean_uri_to_vec(self.path.clone());

        if self.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        return self.parameters_route_match(route_path, request_path);
    }

    
    fn parameters_route_match(&mut self, route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
        let mut params: Values = Values::new();

        for (i, seg) in route_path.iter().enumerate() {
            if i > request_path.len() - 1 {
                return (false, Values::new());
            }

            let seg_match = request_path[i].clone();

            if seg == "*" {
                return (true, Values::new());
            }

            if seg == &seg_match {
                continue;
            }

            if PARAM_REGEX.is_match(&seg.to_string()) {
                params.insert(seg.trim_start_matches('{').trim_end_matches('}').to_owned(), seg_match);

                continue;
            }

            return (false, Values::new());
        }

        return (true, params)
    }

    pub fn handle_middlewares(&mut self, req: &'r mut Request, res: &'r mut Response, middlewares: &Middlewares) -> Option<&'r mut Response> {
        for middleware_ref in  &self.middlewares {
            let middleware = middlewares.get(middleware_ref).unwrap();
            let mut next = Next::new();

            middleware(req, res, &mut next);

            if !next.is_move {
                return None;
            }
        }

        return Some(res);
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

impl <'r>Router<'r> {
    pub fn get<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("GET", path, callback, middleware);
    }

    pub fn post<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        
    {
        self.route("POST", path, callback, middleware);
    }

    pub fn patch<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn put<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("PUT", path, callback, middleware);
    }

    pub fn delete<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("DELETE", path, callback, middleware);
    }   

    pub fn options<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn head<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.route("CONNECT", path, callback, middleware);
    }

    pub fn route<C>(&mut self, method: &str, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
    {
        let path = self.path(path).join("/");
        let resolved = self.merge_middlewares(middleware);

        self.router.add_web_route(method, path, callback, resolved);
    }

    pub fn not_found<C>(&mut self, callback: C)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.router.not_found_callback = Some(Box::new(move |req, res| block_on(callback(req, res))));
    }

    pub fn ws<C>(&mut self, path: &str, callback: C, middleware: Option<Vec<Middleware>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Ws), Output = ()> + Send + Sync + 'static,
    {
        let path = self.path(path).join("/");
        let resolved = self.merge_middlewares(middleware);

        self.router.ws.push(Route{
            path: path,
            method: "GET".to_string(),
            route: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: resolved,
        });
    }

    pub fn group<'g>(&'g mut self , path: &str, group: Group, middleware: Option<Vec<Middleware>>) {
        group(Router{
            path: self.path(path),
            middleware: self.merge_middlewares(middleware),
            router: self.router,
        });
    }

    fn merge_middlewares(&mut self, middlewares: Option<Vec<Middleware>>) -> MiddlewaresRef {
        let mut resolved = self.middleware.clone();

        resolved.extend(self.resolve_middlewares(middlewares.or(Some(vec![])).unwrap()));

        return resolved;
    }

    pub fn resolve_middlewares(&mut self, middlewares: Vec<Middleware>) -> MiddlewaresRef {
        let mut resolved: MiddlewaresRef = vec![];

        for mut middleware in middlewares {
            let address = format!("{:?}", &mut middleware);

            if self.router.middlewares.get(&address).is_some() {
                resolved.push(address);

                continue;
            }

            // self.router.middlewares.insert(address.clone(), Box::new(move |req, res, next| block_on(middleware(req, res, next))));
            self.router.middlewares.insert(address.clone(), Box::new(middleware));

            resolved.push(address);
        }

        return resolved;
    }

    fn path(&mut self, path: &str) -> Vec<String> {
        return merge(vec![self.path.clone(), vec![path.to_string()]]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }
}