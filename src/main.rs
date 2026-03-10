use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use async_std::task::block_on;


pub struct Request {
}

pub struct Response {
}

pub struct Ws {
}

pub(crate) type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync;
pub(crate) type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync;
pub(crate) type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws);

pub struct Route<Handler: ?Sized> {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) handler: Box<Handler>,
    pub(crate) middlewares: Vec<String>
}

impl<Handler: ?Sized> std::fmt::Debug for Route<Handler> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .finish()
    }
}

pub struct Server {
    web_routes: Vec<Route<WebRoute>>,
    ws_routes: Vec<Route<WsRoute>>,
    middlewares: HashMap<String, Box<Middleware>>,
}


impl Server {
    pub(crate) fn new() -> Self {
        Self {
            web_routes: Vec::new(),
            ws_routes: Vec::new(),
            middlewares: HashMap::new(),
        }
    }

    pub fn router(&'_ mut self) -> Router<'_> {
        return Router {
            path: String::new(),
            server: self,
            middlewares: Vec::new(),
        };
    }

    pub(crate) fn register_middleware<C>(&mut self, callback: C) -> String
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        let r#ref = format!("{:p}", &callback);

        self.middlewares.insert(r#ref.clone(), Box::new(move |req, res, next| block_on(callback(req, res, next))));

        return r#ref;
    }

    pub(crate) fn on_request<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        return Ok((req, res))
    }
}

pub struct Router<'q> {
    pub(crate) path: String,
    pub(crate) server: &'q mut Server,
    pub(crate) middlewares: Vec<String>
}

pub fn join_url(url: Vec<String>) -> String {
    return url.iter()
        .map(|u| String::from(u.trim_matches('/'))).collect::<Vec<_>>()
        .join("/")
        .trim_matches('/')
        .to_string();
}

impl <'q>Router<'q> {
    pub fn get<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        self.route("GET", path, callback);
    }

    pub fn post<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static,
    {
        self.route("POST", path, callback);
    }

    pub fn route<C>(&mut self, method: &str, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static
    {
        self.server.web_routes.push(Route {
            method: String::from(method.to_uppercase()),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: self.middlewares.clone(),
        });
    }
    
    pub fn group(&mut self, path: &str, group: Group) {
        group(&mut Router {
            path: join_url(vec![self.path.clone(), String::from(path)]),
            server: self.server,
            middlewares: self.middlewares.clone(),
        });
    }

    pub fn ws<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Ws) -> &'a mut Response + Send + Sync + 'static
    {
        // self.route("POST", path, callback);
    }


    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        // self.middlewares
        //     .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));

        return self;
    }

}


pub struct Next {
    pub(crate) is_move: bool,
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



pub(crate) type Group = for<'a> fn(&mut Router);

fn main() {
    let mut server = Server::new();

    server.router()
        .middleware(async |req, res, next | {
            return next.handle(res);
        })
        .group("api", |router| {
            router.get("/", async |req, res| {
                return res
            });
            router.group("users", |router| {
                router.get("/", async |req, res| {
                    return res
                });
                router.group("{id}", |router| {
                    router.get("/", async |req, res| {
                        return res
                    });
                });
            });
        });

    println!("{:?}", server.web_routes);


    println!("Hello, world!")
}