use async_std::task::block_on;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::runtime::Handle;

use url_domain_parse::Url;

use crate::{request::Request, response::Response, router::{middleware::register, next::Next}, utils::{Values, url::uri_to_vec}, ws::Ws};

pub(crate) mod resolver;
pub(crate) mod middleware;
pub(crate) mod routes;
pub(crate) mod next;

pub(crate) type Middleware = dyn for<'a> Fn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static;
pub(crate) type WebRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Response) -> &'a mut Response + Send + Sync + 'static;
pub(crate) type WsRoute = dyn for<'a> Fn(&'a mut Request, &'a mut Ws) + Send + Sync + 'static;
pub(crate) type Group = for<'a> fn(&mut Router);


static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex"));

pub(crate) struct Route<Handler: ?Sized> {
    pub subdomain: String,
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) handler: Box<Handler>,
    pub(crate) middlewares: Vec<String>,
}


impl <Handler: ?Sized>Route<Handler> {

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static,
    {
        self.middlewares
            .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));

        return self;
    }

    pub(crate) fn is_match<'a>(&self, req: &'a mut Request) -> (bool, Values) {
        if self.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        return self.parameters_route_match(req);
    }

    fn dymanic_parameter_match(&self, path: String, matching: String) -> Option<(String, String)> {
        if PARAM_REGEX.is_match(&path) == false {
            return None;
        }

        return Some((path.trim_start_matches('{').trim_end_matches('}').to_owned(), matching));
    }
    
    // TODO: refactor
    fn parameters_route_match<'a>(&self, req: &'a mut Request) -> (bool, Values) {
        let mut parameters= Values::new();
        let route_path= uri_to_vec(self.path.clone());
        let request_path = uri_to_vec(req.path.clone());
        // TODO: fix plugin Domain parser.
        let url_result = Url::parse(format!("http://{}", &req.host).as_str());

        if url_result.is_err() {
            return (false, Values::new());
        }

        let url = url_result.unwrap();
        let subdomain_route: Vec<String> = self.subdomain.split(".").map(|v| v.to_string()).collect();
        let subdomain_req: Vec<String> = url.subdomain().or(Some(String::new())).unwrap().split(".").map(|v| v.to_string()).collect();

        for (i, _) in request_path.iter().enumerate() {
            if subdomain_route.len() != subdomain_req.len() {
                return (false, Values::new());
            }

            for (j, _ )in subdomain_route.iter().enumerate() {
                if subdomain_route[j] == subdomain_req[j] {
                    continue;
                }

                if subdomain_req[j] == "" {
                    return (false, Values::new());
                }

                if let Some((k, v)) = self.dymanic_parameter_match(subdomain_route[j].clone(), subdomain_req[j].clone()) {                    
                    parameters.insert(k, v);

                    continue;
                }

                return (false, Values::new());
            }

            if i > route_path.len() - 1 {
                return (false, Values::new());
            }

            if route_path[i] == "*" {
                return (true, parameters);
            }

            if route_path[i] == request_path[i] {
                // Off guard
                if request_path.len() - 1 == i && route_path.len() > request_path.len() {
                    return (false, Values::new());
                }

                continue;
            }

            if let Some((k, v)) = self.dymanic_parameter_match(route_path[i].clone(), request_path[i].clone()) {
                parameters.insert(k, v);

                continue;
            }

            // Off guard
            if request_path.len() - 1 == i && route_path.len() > request_path.len() {
                return (false, Values::new());
            }

            return (false, Values::new());
        }

        return (true, parameters)
    }
}



impl<'q, Handler: ?Sized> std::fmt::Debug for Route<Handler> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("middlewares", &self.middlewares)
            .finish()
    }
}

pub struct Router {
    pub(crate) web_routes: Vec<Box<Route<WebRoute>>>,
    pub(crate) ws_routes: Vec<Box<Route<WsRoute>>>,
    pub(crate) path: String,
    pub(crate) middlewares: Vec<String>,
    pub(crate) group: Option<Group>,
    pub(crate) routers: Vec<Box<Router>>,
}

pub fn join_url(url: Vec<String>) -> String {
    return url.iter()
        .map(|u| String::from(u.trim_matches('/'))).collect::<Vec<_>>()
        .join("/")
        .trim_matches('/')
        .to_string();
}

impl Router {
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
        self.web_routes.push(Box::new(Route {
            subdomain: String::new(),
            method: String::from(method.to_uppercase()),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, res| block_on(callback(req, res))),
            middlewares: self.middlewares.clone()
        }));
    }
    
    pub fn group<'g>(&'g mut self, path: &str, group: Group) -> GroupRouter<'g> {
        let idx = self.routers.len();

        self.routers.push(Box::new(Router {
            web_routes: Vec::new(),
            ws_routes: Vec::new(),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            middlewares: self.middlewares.clone(),
            group: Some(group),
            routers: vec![],
        }));

        return GroupRouter::new(self.routers[idx].as_mut())
    }

    pub fn ws<C>(&mut self, path: &str, callback: C)
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Ws) + Send + Sync + 'static
    {
        self.ws_routes.push(Box::new(Route {
            subdomain: String::new(),
            method: String::from("GET"),
            path: join_url(vec![self.path.clone(), String::from(path)]),
            handler: Box::new(move |req, ws| block_on(callback(req, ws))),
            middlewares: vec![],
        }));
    }


    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        self.middlewares
            .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));
        return self;
    }

}


pub struct GroupRouter<'g> {
    router: &'g mut Router
}

impl <'g>GroupRouter<'g> {
    pub(crate) fn new(router: &'g mut Router) -> Self {
        return Self {
            router: router
        }
    }

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn(&'a mut Request, &'a mut Response, &'a mut Next) -> &'a mut Response + Send + Sync + 'static
    {
        self.router.middleware(callback);
        return self;
    }
}
