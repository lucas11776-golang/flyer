pub mod group;

use futures::executor::block_on;
use futures_util::future::BoxFuture;
use futures::future::{Future};

use crate::{router::group::GroupRouter};
use crate::utils::merge;
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::clean_url;

pub type WebRoute = dyn for<'a> Fn(Request, Response) -> Response + Send + Sync;
pub type WsRoute<'a> = dyn Fn(Request, Ws) -> BoxFuture<'static, ()> + Send + Sync;

pub type Middleware = for<'a>  fn (req: Request, res: Response, next: Next<'a>) -> Response;
pub type Middlewares = Vec<Middleware>;
pub type Group<'s> = fn (router: Router);

pub struct Next<'a> {
    is_next: &'a mut bool,
    response: &'a mut Response,
}

pub struct Router<'r> {
    pub(crate) router: &'r mut GroupRouter,
    pub(crate) path: Vec<String>,
    pub(crate) middleware: Middlewares,
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

impl <'r>Router<'r> {
    pub fn get<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("GET", path, callback, middleware);
    }   
 
    pub fn post<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("POST", path, callback, middleware);
    }

    pub fn patch<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn put<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("PUT", path, callback, middleware);
    }

    pub fn delete<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("DELETE", path, callback, middleware);
    }

    pub fn head<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("CONNECT", path, callback, middleware);
    }

    pub fn options<C>(&mut self, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn route<C>(&mut self, method: &str, path: &str, callback: C, middleware: Option<Middlewares>)
    where
        C: for<'a> AsyncFn<(Request, Response), Output = Response> + Send + Sync + 'static
    {
        let middlewares = self.merge_middlewares(middleware);

        self.router.add_web_route(
            method,
            // TODO: fix
            Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
            callback,
            middlewares
        );
    }

    fn merge_middlewares(&mut self, middleware: Option<Middlewares>) -> Middlewares {
        let mut middlewares: Middlewares = self.middleware.clone();

        if middleware.is_some() {
            middlewares = merge(vec![middlewares, middleware.unwrap()]);
        }

        return middlewares;
    }

    pub fn ws<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Ws) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + Sync + 'static,
    {
        let middlewares = self.merge_middlewares(middleware);

        // self.router.ws.push(Route{
        //     // TODO: fix
        //     path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
        //     method: "GET".to_owned(),
        //     route: Box::new(move |req: Request, ws: Ws | callback(req, ws).boxed()),
        //     middlewares: middlewares,
        // });
    }

    pub fn group<'s>(&'s mut self , path: &str, group: Group<'s>, middleware: Option<Middlewares>) {
        group(Router{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]),
            middleware: self.merge_middlewares(middleware),
            router: self.router,
        });
    }

    pub fn not_found<R, F>(&mut self, callback: R)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.router.not_found_callback = Some(Box::new(move |req: Request, res: Response| block_on(callback(req, res))));
    }

    fn get_path(old: Vec<String>, new: Vec<String>) -> Vec<String> {
        return merge(vec![old,new]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }
}