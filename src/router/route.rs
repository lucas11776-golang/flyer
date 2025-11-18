use regex::Regex;
use once_cell::sync::Lazy;
use futures::executor::block_on;

use crate::router::{Middleware, Next, Router};
use crate::utils::Values;
use crate::request::Request;
use crate::response::Response;
use crate::utils::url::uri_to_vec;

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex"));

pub struct GroupRoute<'r> {
    pub(crate) router: &'r mut Box<Router>
}

pub struct Route<R> {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub(crate) middlewares: Vec<Box<Middleware>>,
}

impl <'r>GroupRoute<'r> {
    pub(crate) fn new(router: &'r mut Box<Router>) -> GroupRoute<'r> {
        return Self {
            router: router,
        }
    }

    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.router.middlewares.push(Box::new(move |req, res, next| block_on(callback(req, res, next))));

        return self;
    }
}

impl <'r, R> Route<R> {
    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.middlewares.push(Box::new(move |req, res, next| block_on(callback(req, res, next))));

        return self;
    }

    pub(crate) fn is_match(&mut self, req: &'r mut Request) -> (bool, Values) {
        let request_path: Vec<String> = uri_to_vec(req.path.clone());
        let route_path: Vec<String> = uri_to_vec(self.path.clone());

        if self.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        return self.parameters_route_match(route_path, request_path);
    }
    
    fn parameters_route_match(&mut self, route_path: Vec<String>, request_path: Vec<String>) -> (bool, Values) {
        let mut params: Values = Values::new();

        for (i, _) in request_path.iter().enumerate() {
            if i > route_path.len() - 1 {
                return (false, Values::new());
            }

            if route_path[i] == "*" {
                return (true, params);
            }

            if route_path[i] == request_path[i] {
                // Off guard
                if request_path.len() - 1 == i && route_path.len() > request_path.len() {
                    return (false, Values::new());
                }

                continue;
            }

            if PARAM_REGEX.is_match(&route_path[i].to_string()) {
                params.insert(route_path[i].trim_start_matches('{').trim_end_matches('}').to_owned(), request_path[i].to_string());

                continue;
            }

            // Off guard
            if request_path.len() - 1 == i && route_path.len() > request_path.len() {
                return (false, Values::new());
            }

            return (false, Values::new());
        }

        return (true, params)
    }
}
