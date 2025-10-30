pub mod group;

use futures_util::future::BoxFuture;
use futures::future::{Future, FutureExt};

use crate::{router::group::GroupRouter};
use crate::utils::merge;
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::clean_url;

pub type WebRoute<'a> = dyn Fn(Request, Response) -> BoxFuture<'static, Response> + Send + Sync;
pub type Middleware = for<'a>  fn (req: Request, res: Response, next: Next<'a>) -> Response;
pub type Middlewares = Vec<Middleware>;
pub type WsRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Ws);
pub type Group<'s> = fn (router: Router);

pub struct Next<'a> {
    is_next: &'a mut bool,
    response: &'a mut Response,
}

pub struct Router<'a> {
    pub(crate) router: &'a mut GroupRouter,
    pub(crate) path: Vec<String>,
    pub(crate) middleware: Middlewares,
}

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Middlewares
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
        self.route("GET", path, callback, middleware);
    }   
 
    pub fn post<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.route("POST", path, callback, middleware);
    }

    pub fn patch<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn put<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.route("PUT", path, callback, middleware);
    }

    pub fn delete<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.route("DELETE", path, callback, middleware);
    }

    pub fn head<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.route("CONNECT", path, callback, middleware);
    }

    pub fn options<R, F>(&mut self, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static,
    {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn route<R, F>(&mut self, method: &str, path: &str, callback: R, middleware: Option<Middlewares>)
    where
        R: Fn(Request, Response) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + Sync + 'static
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

    pub fn ws(&mut self, path: &str, callback: WsRoute, middleware: Option<Middlewares>) {
        let middlewares = self.merge_middlewares(middleware);

        self.router.ws.push(Route{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
            method: "GET".to_owned(),
            route: callback,
            middlewares: middlewares,
        });
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
        self.router.not_found_callback = Some(Box::new(move |req: Request, res: Response| callback(req, res).boxed()));
    }

    fn get_path(old: Vec<String>, new: Vec<String>) -> Vec<String> {
        return merge(vec![old,new]).iter()
            .map(|x| clean_url(x.to_owned()))
            .filter(|x| x != "")
            .collect();
    }
}