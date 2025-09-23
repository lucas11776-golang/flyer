pub mod group;

use std::io::Result;

use crate::router::group::GroupRouter;
use crate::utils::merge;
use crate::ws::Ws;

use crate::request::Request;
use crate::response::Response;
use crate::utils::url::clean_url;

// TODO: route must be async...
pub type WebRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Response) -> &'a mut Response;
pub type Middleware = for<'a>  fn (req: &'a mut Request, res: &'a mut Response, next: &'a mut Next<'a>) -> &'a mut Response;
pub type Middlewares = Vec<Middleware>;
pub type WsRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Ws);
pub type Group = fn (router: &mut Router);

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
    pub(crate) middlewares: Middlewares,
}

impl <'a>Next<'a> {
    pub fn next(&'a mut self) -> &'a mut Response {
        *self.is_next = true;

        return &mut self.response;
    }
}

impl <'a>Router<'a> {
    pub fn get(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("GET", path, callback, middleware);
    }   
 
    pub fn post(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("POST", path, callback, middleware);
    }

    pub fn patch(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn put(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("PUT", path, callback, middleware);
    }

    pub fn delete(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("DELETE", path, callback, middleware);
    }

    pub fn head(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("CONNECT", path, callback, middleware);
    }

    pub fn options(&mut self, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn route(&mut self, method: &str, path: &str, callback: WebRoute, middleware: Option<Middlewares>) {
        self.add_web_route(method, path, callback, middleware).unwrap();
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

    pub fn group(&mut self , path: &str, group: Group, middleware: Option<Middlewares>) {
        let middlewares = self.get_middlewares(middleware);

        group(&mut Router{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]),
            router: self.router,
            middleware: middlewares
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

    fn add_web_route(&mut self, method: &str, path: &str, callback: WebRoute, middleware: Option<Middlewares>) -> Result<()> {
        let middlewares = self.get_middlewares(middleware);

        self.router.web.push(Route{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
            method: method.to_string(),
            route: callback,
            middlewares: middlewares,
        });

        Ok(())
    }
}