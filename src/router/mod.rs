pub mod group;

use std::collections::HashMap;
use std::mem::take;
use std::mem::copy;

use futures::executor::block_on;
use futures_util::future::BoxFuture;
use futures::future::{Future};

use crate::{router::group::GroupRouter};
use crate::utils::merge;
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::clean_url;

pub type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync;
pub type WsRoute<'a> = dyn Fn(Request, Ws) -> BoxFuture<'static, ()> + Send + Sync;

// pub type Middleware = for<'a>  fn (req: Request, res: Response, next: Next) -> Response;

// pub type MiddlewareT = dyn for<'a> Fn(&'a mut Request, &'a mut Response, Next) -> &'a mut Response + Send + Send + 'static;

// pub type MiddlewareT = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> dyn Future<Output = &'a mut Response>;

pub type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static;

// pub type Middlewares = Vec<Middleware>;

pub type Middlewares = HashMap<String, Box<Middleware>>;

pub type MiddlewaresRef = Vec<String>;


pub type Group<'s> = fn (router: Router);


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
    pub fn get<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("GET", path, callback, middleware);
    }

    pub fn post<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("POST", path, callback, middleware);
    }

    pub fn patch<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn put<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("PUT", path, callback, middleware);
    }

    pub fn delete<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("DELETE", path, callback, middleware);
    }   

    pub fn options<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn head<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("CONNECT", path, callback, middleware);
    }

    pub fn route<C, M>(&mut self, method: &str, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
    {
        let path = self.get_path_v2(path).join("/");
        let resolved = self.merge_middlewares(middleware);

        self.router.add_web_route(method, path, callback, resolved);
    }

    pub fn not_found<C>(&mut self, callback: C)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.router.not_found_callback = Some(Box::new(move |req, res| block_on(callback(req, res))));
    }

    pub fn ws<R, F>(&mut self, path: &str, callback: R, middleware: Option<()>)
    where
        R: Fn(Request, Ws) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        // TODO: implement
    }

    pub fn group<'s, M>(&'s mut self , path: &str, group: Group<'s>, middleware: Option<Vec<M>>)   
    where
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static
     {
        group(Router{
            // TODO: fix
            path: self.get_path_v2(path),
            middleware: self.merge_middlewares(middleware),
            router: self.router,
        });
    }

    fn merge_middlewares<M>(&mut self, middlewares: Option<Vec<M>>) -> MiddlewaresRef
    where
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + 'static + Send + Sync
    {
        // TODO: find way to 
        let mut resolved = self.middleware.clone();

        resolved.extend(self.resolve_middlewares(middlewares.or(Some(vec![])).unwrap()));

        return resolved;
    }

    pub fn resolve_middlewares<M>(&mut self, middlewares: Vec<M>) -> MiddlewaresRef
    where
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + 'static + Send + Sync
    {
        let mut resolved: MiddlewaresRef = vec![];

        for mut middleware in middlewares {
            let address = format!("{:?}", &mut middleware as *mut M);

            if self.router.middlewares.get(&address).is_some() {
                resolved.push(address);

                continue;
            }

            self.router.middlewares.insert(address.clone(), Box::new(move |req, res, next| block_on(middleware(req, res, next))));

            resolved.push(address);
        }

        return resolved;
    }

    fn get_path_v2(&mut self, path: &str) -> Vec<String> {
        return merge(vec![self.path.clone(), vec![path.to_string()]]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }
}