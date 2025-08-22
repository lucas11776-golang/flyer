use crate::{
    request::{Request, Values},
    response::{Response},
    utils::url::{self, clean_url}
};

pub type WebRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Response) -> &'a mut Response;
pub type Next<'a> = fn () -> &'a mut Response;
pub type Middleware<'a> = &'a mut fn (req: Request, res: Response, next: Next<'a>) -> &'a mut Response;
pub type Middlewares<'a> = Vec<Middleware<'a>>; // TODO: find better way...

pub struct GroupRouter {
    web: Vec<Route<WebRoute>>,
    pub(crate) not_found_callback: Option<WebRoute>,
}

impl GroupRouter {
    pub fn match_web_routes<'a>(&mut self, req: &mut Request, res: &'a mut Response) -> Option<&'a mut Response> {
        for route in &mut self.web {
            let (matches, parameters) = Router::match_route(route, req);

            if !matches {
                continue;
            }
            
            req.parameters = parameters;

            (route.route)(req, res);

            return Some(res);
        }

        return None;
    }
}

pub struct Router<'a> {
    pub(crate) router: &'a mut GroupRouter,
    pub(crate) path: Vec<String>,
    
}

pub fn new_group_router<'a>() -> GroupRouter {
    return GroupRouter {
        web: vec![],
        not_found_callback: None
    }
}

type Group = fn (router: &mut Router);

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Vec<Middleware<'static>>,
}

use regex::Regex;
use std::{collections::HashMap, io::Result};
use once_cell::sync::Lazy;

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex")
});


use std::net::IpAddr;

fn get_subdomain(host: &str) -> String {
    let host = host.split(':').next().unwrap_or("");

    if host.parse::<IpAddr>().is_ok() {
        return String::new();
    }

    let parts: Vec<&str> = host.split('.').collect();

    if parts.len() < 3 {
        return String::new();
    }

    parts[..parts.len() - 2].join(".")
}

pub fn merge<T>(items: Vec<Vec<T>>) -> Vec<T> {
    let mut merged: Vec<T> = vec![];

    for item in items {
        merged.extend(item);
    }

    return merged
}

impl <'a>Router<'a> {
    pub fn group(&mut self , path: &str, group: Group) {
        group(&mut Router{
            // TODO: fix
            path: Router::get_path(self.path.clone(), vec![path.to_string()]),
            router: self.router
        });
    }

    pub fn get(&mut self, path: &str, callback: WebRoute, middleware: Option<&'a mut Vec<Middleware>>) {
        self.route("GET", path, callback, middleware);
    }

    pub fn post(&mut self, path: &str, callback: WebRoute, middleware: Option<&'static mut Vec<Middleware>>) {
        self.route("POST", path, callback, middleware);
    }

    pub fn patch(&mut self, path: &str, callback: WebRoute, middleware: Option<&'static mut Vec<Middleware>>) {
        self.route("PATCH", path, callback, middleware);
    }

    pub fn put(&mut self, path: &str, callback: WebRoute, middleware: Option<&'static mut Vec<Middleware>>) {
        self.route("PUT", path, callback, middleware);
    }

    pub fn delete(&mut self, path: &str, callback: WebRoute, middleware: Option<&'static mut Vec<Middleware>>) {
        self.route("DELETE", path, callback, middleware);
    }

    pub fn head(&mut self, path: &str, callback: WebRoute, middleware: Option<&'static mut Vec<Middleware>>) {
        self.route("CONNECT", path, callback, middleware);
    }

    pub fn options(&mut self, path: &str, callback: WebRoute, middleware: Option<&'static mut Vec<Middleware>>) {
        self.route("OPTIONS", path, callback, middleware);
    }

    pub fn route(&mut self, method: &str, path: &str, callback: WebRoute, middleware: Option<&'a mut Vec<Middleware>>) {
        self.add_web_route(method, path, callback, middleware).unwrap();
    }

    pub fn not_found(&mut self, callback: WebRoute) {
        self.router.not_found_callback = Some(callback);
    }

    fn parameters_route_match(route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
        let mut params: Values = HashMap::new();

        for (i, seg) in request_path.iter().enumerate() {
            if i > route_path.len() - 1 {
                return (false, Values::new());
            }

            let route_seg = route_path[i].clone();

            if route_seg == "*" {
                return (true, Values::new());
            }

            if seg == &route_seg {
                continue;
            }

            if PARAM_REGEX.is_match(&route_seg.to_string()) {
                let key = route_seg.trim_start_matches('{').trim_end_matches('}');

                params.insert(key.to_string(), (*seg).to_string());

                continue;
            }

            return (false, Values::new());
        }

        return (true, params)
    }

    fn get_path(old: Vec<String>, new: Vec<String>) -> Vec<String> {
        return merge(vec![old,new]).iter()
            .map(|x| clean_url(x.to_owned())).filter(|x| x != "")
            .collect();
    }

    fn add_web_route(&mut self, method: &str, path: &str, callback: WebRoute, middleware: Option<&'a mut Vec<Middleware>>) -> Result<()> {
        match middleware {

            
            Some(middleware) => {
                self.router.web.push(Route{
                    path: url::clean_url(path.to_string()),
                    method: method.to_string(),
                    route: callback,
                    // middlewares: middleware,
                    middlewares: vec![],
                });
            },
            None => {
                self.router.web.push(Route{
                    // TODO: fix
                    path: Router::get_path(self.path.clone(), vec![path.to_string()]).join("/"),
                    method: method.to_string(),
                    route: callback,
                    middlewares: vec![],
                });
            },
        }

        Ok(())
    }

    fn match_route<T>(route: &mut Route<T>, req: &mut Request) -> (bool, Values) {
        let request_path: Vec<String> = url::clean_uri_to_vec(req.path.clone());
        let route_path: Vec<String> = url::clean_uri_to_vec(route.path.clone());

        if route.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        let (matches, parameters) = Router::parameters_route_match(route_path, request_path);

        if !matches {
            return (false, Values::new());
        }

        return (true, parameters);
    }

}