pub mod group;

use std::io::Result;

use futures_util::future::BoxFuture;

use crate::{router::group::GroupRouter, HTTP};
use crate::utils::merge;
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::clean_url;




use futures::future::{Future, FutureExt};



pub type TRoute<'a> = dyn Fn(Request, Response) -> BoxFuture<'static, Response> + Send + Sync; // TODO: 'a --


// TODO: current option
// pub type TRoute<'a> = dyn Fn(&'a mut Request, &'a mut Response) -> BoxFuture<'a, &'a mut Response> + Send + Sync;

// TODO: route must be async...
pub type WebRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Response) -> &'a mut Response;
pub type Middleware = for<'a>  fn (req: Request, res: Response, next: Next<'a>) -> Response;
pub type Middlewares = Vec<Middleware>;
pub type WsRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Ws);
pub type Group<'s> = fn (router: Router);

pub struct Next<'a> {
    is_next: &'a mut bool,
    response: &'a mut Response,
}

pub struct Router<'a> {
    // pub(crate) http: &'a mut HTTP<'a>,
    pub(crate) router: &'a mut GroupRouter,
    pub(crate) path: Vec<String>,
    pub(crate) middleware: Middlewares,
    // pub(crate) get: Option<Box<TRoute<'a>>>
}

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Middlewares,
}

impl <'a>Next<'a> {
    pub fn next(&'a mut self) -> &'a mut Response {
        *self.is_next = true;

        return &mut self.response;
    }
}

impl <'a>Router<'a> {
    pub fn get<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.add_web_route("GET", path, callback, middleware);
    }   
 
    pub fn post<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        // self.add_web_route("POST", path, callback, middleware);
    }

    pub fn patch<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        // self.add_web_route("PATCH", path, callback, middleware);
    }

    pub fn put<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        // self.add_web_route("PUT", path, callback, middleware);
    }

    pub fn delete<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        // self.add_web_route("DELETE", path, callback, middleware);
    }

    pub fn head<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        // self.add_web_route("CONNECT", path, callback, middleware);
    }

    pub fn options<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        // self.add_web_route("OPTIONS", path, callback, middleware);
    }

    fn get_middlewares(&mut self, middleware: Option<Middlewares>) -> Middlewares {
        let mut middlewares: Middlewares = self.middleware.clone();

        if middleware.is_some() {
            middlewares = merge(vec![middlewares, middleware.unwrap()]);
        }

        return middlewares;
    }

    pub fn ws(&mut self, path: &str, callback: WsRoute, middleware: Option<Middlewares>) {
        let middlewares = self.get_middlewares(middleware);

        self.router.ws.push(Route{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
            method: "GET".to_owned(),
            route: callback,
            middlewares: middlewares,
        });
    }

    pub fn group<'s>(&'s mut self , path: &str, group: Group<'s>, middleware: Option<Middlewares>)
    where
    {
        group(Router{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]),
            middleware: self.get_middlewares(middleware),
            router: self.router,
        });
    }

    pub fn not_found(&mut self, callback: Box<WebRoute>) {
        self.router.not_found_callback = Some(callback);
    }

    fn get_path(old: Vec<String>, new: Vec<String>) -> Vec<String> {
        return merge(vec![old,new]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }

    fn add_web_route<C, F>(&mut self, method: &str, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        let middlewares = self.get_middlewares(middleware);

        self.router.web.push(Route{
            path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
            method: method.to_string(),
            route: Box::new(move |req: Request, res: Response| callback(req, res).boxed()),
            middlewares: middlewares,
        });

    }
}