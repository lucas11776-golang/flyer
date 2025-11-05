pub mod group;

use futures::SinkExt;
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

pub type Middleware = for<'a>  fn (req: Request, res: Response, next: Next<'a>) -> Response;

pub type MiddlewareT = dyn for<'a> Fn(&'a mut Request, &'a mut Response, Next<'a>) -> &'a mut Response + Send + Send + 'static;

pub type Middlewares = Vec<Middleware>;
pub type Group<'s> = fn (router: Router);

pub struct Next<'a> {
    is_next: &'a mut bool,
    response: &'a mut Response,
}

pub struct Router<'r> {
    pub(crate) router: &'r mut GroupRouter,
    pub(crate) path: Vec<String>,
    pub(crate) middleware: Vec<Box<MiddlewareT>>,
}

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Vec<Box<MiddlewareT>>,
}

impl <'a>Next<'a> {
    pub fn next(&'a mut self) -> &'a mut Response {
        *self.is_next = true;

        return &mut self.response;
    }
}

impl <'r>Router<'r> {
    pub fn delete<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("DELETE", path, callback, middleware);
    }   
 
    pub fn get<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("GET", path, callback, middleware);
    }

    fn get_path(old: Vec<String>, new: Vec<String>) -> Vec<String> {
        return merge(vec![old,new]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }

    pub fn group<'s, M>(&'s mut self , path: &str, group: Group<'s>, middleware: Option<Vec<M>>)   
    where
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
     {

        let mut resolved: Vec<Box<MiddlewareT>> = vec![];

        if middleware.is_some() {
            resolved.extend(self.merge_middlewares(middleware.unwrap()));
        } 

        group(Router{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]),
            middleware: resolved,
            router: self.router,
        });
    }

    // pub fn head<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    // where
    //     C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static
    // {
    //     self.route("CONNECT", path, callback, middleware);
    // }

    // fn merge_middlewares(&mut self, middleware: Option<Middlewares>) -> Middlewares {
    //     let mut middlewares: Middlewares = self.middleware.clone();

    //     if middleware.is_some() {
    //         middlewares = merge(vec![middlewares, middleware.unwrap()]);
    //     }

    //     return middlewares;
    // }

    fn merge_middlewares<M>(&mut self, middlewares: Vec<M>) -> Vec<Box<MiddlewareT>>
    where
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + 'static + Send + Sync
    {

        return vec![];
    }

    pub fn not_found<C>(&mut self, callback: C)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.router.not_found_callback = Some(Box::new(move |req, res| block_on(callback(req, res))));
    }


    pub fn resolve_middlewares<M>(&mut self, middlewares: Vec<M>) -> Vec<Box<MiddlewareT>>
    where
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + 'static + Send + Sync
    {
        let mut resolved: Vec<Box<MiddlewareT>> = vec![];

        for middleware in middlewares {
            // resolved.push(Box::new(|req, res, next| block_on(middleware(req, res, next))));
        }

        return resolved;
    }

    pub fn options<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn patch<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn post<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("POST", path, callback, middleware);
    }

    pub fn put<C, M>(&mut self, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {
        self.route("PUT", path, callback, middleware);
    }

    pub fn route<C, M>(&mut self, method: &str, path: &str, callback: C, middleware: Option<Vec<M>>)
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response), Output = &'a mut Response> + Send + Sync + 'static,
        M: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, Next<'a>), Output = &'a mut Response> + Send + Sync + 'static
    {

        let mut resolved: Vec<Box<MiddlewareT>> = vec![];


        if middleware.is_some() {
            resolved.extend(self.merge_middlewares(middleware.unwrap()));
        }

        self.router.add_web_route(
            method,
            // TODO: fix
            Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
            callback,
            resolved
        );
    }

    pub fn ws<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Ws) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        // let middlewares = self.merge_middlewares(middleware);

        // self.router.ws.push(Route{
        //     // TODO: fix
        //     path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
        //     method: "GET".to_owned(),
        //     route: Box::new(move |req: Request, ws: Ws | callback(req, ws).boxed()),
        //     middlewares: middlewares,
        // });
    }
}