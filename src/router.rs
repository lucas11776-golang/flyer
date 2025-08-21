use crate::{request::{Request, Values}, response::Response, utils::url};

pub type WebRoute = for<'a> fn (req: &'a mut Request, res: &'a mut Response) -> &'a mut Response;
pub type Next = fn () -> Response;
pub type Middleware = fn (req: Request, res: Response, next: Next) -> Response;

pub struct Router {
    path: Vec<String>,
    pub(crate) web_routes: Vec<Route<WebRoute>>,
    pub(crate) not_found_callback: Option<WebRoute>,
    router: Option<&'static mut Pin<&'static mut Router>>
    
}

pub fn new_router() -> Router {
    return Router {
        path: [].into(),
        web_routes: vec![],
        not_found_callback: None,
        router: None
    }
}

type Group = fn (router: &mut Router);

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Vec<Middleware>,
}

use regex::Regex;
use std::{any::Any, collections::HashMap, pin::{self, pin, Pin}};
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

impl Router {
    pub fn group(&mut self , path: &str, group: Group) {
        let g = Router{
            path: merge(vec![self.path.clone(), vec![path.to_owned()]]),
            web_routes: vec![],
            not_found_callback: None,
            router: None
        };
    }

    pub fn get(&mut self, path: &str, callback: WebRoute) {
        self.add_web_route("GET".to_owned(), path, vec![], callback);
    }

    pub fn post(&mut self, path: &str, callback: WebRoute) {
        self.add_web_route("POST".to_owned(), path, vec![], callback);
    }

    pub fn patch(&mut self, path: &str, callback: WebRoute) {
        self.add_web_route("PATCH".to_owned(), path, vec![], callback);
    }

    pub fn put(&mut self, path: &str, callback: WebRoute) {
        self.add_web_route("PUT".to_owned(), path, vec![], callback);
    }

    pub fn delete(&mut self, path: &str, callback: WebRoute) {
        self.add_web_route("DELETE".to_owned(), path, vec![], callback);
    }

    pub fn route(&mut self, method: String, path: &str, callback: WebRoute) {
        self.add_web_route(method, path, vec![], callback);
    }

    pub fn not_found(&mut self, callback: WebRoute) {
        self.not_found_callback = Some(callback);
    }

    fn parameters_route_match(route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
        let mut params: Values = HashMap::new();

        for (i, seg) in request_path.iter().enumerate() {
            if i >= route_path.len() {
                return (false, params);
            }

            let route_seg = route_path[i].clone();

            if route_seg == "*" {
                return (true, params);
            }

            if seg == &route_seg {
                continue;
            }

            if PARAM_REGEX.is_match(&route_seg.to_string()) {
                let key = route_seg.trim_start_matches('{').trim_end_matches('}');
                params.insert(key.to_string(), (*seg).to_string());

                continue;
            }

            return (false, params);
        }

        return (true, params)
    }

    fn add_web_route(&mut self, method: String, path: &str, middleware: Vec<Middleware>, callback: WebRoute) {
        // TODO: find better way...
        match &self.router {
            Some(_router) => {
                // Will fail because is ref -> &...


                
            },
            None => {
                self.web_routes.push(Route {
                    path: url::clean_url(path.to_string()),
                    method: method,
                    route: callback,
                    middlewares: middleware,
                });
            },
        };
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

    pub fn match_web_routes(&mut self, req: &mut Request) -> Option<&Route<WebRoute>> {
        for route in &mut self.web_routes {
            let (matches, parameters) = Router::match_route(route, req);

            if !matches {
                continue;
            }
            
            req.parameters = parameters;

            return Some(route);
        }

        return None;
    }
}