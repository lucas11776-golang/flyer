use async_std::task::block_on;
use regex::Regex;
use once_cell::sync::Lazy;

use crate::router::middleware::register;
use crate::router::{MiddlewaresPointers, Next, Router};
use crate::utils::Values;
use crate::request::Request;
use crate::response::Response;
use crate::utils::url::{parse_host, uri_to_vec};

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex"));

pub struct GroupRoute<'r> {
    pub(crate) router: &'r mut Box<Router>
}

pub struct Route<R> {
    pub subdomain: String,
    pub path: String,
    pub(crate) method: String,
    pub(crate) route: R,
    pub middlewares: MiddlewaresPointers,
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
        self.router
            .middlewares
            .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));

        return self;
    }
}


impl <'r, R> Route<R> {
    pub fn middleware<C>(&mut self, callback: C) -> &mut Self
    where
        C: for<'a> AsyncFn<(&'a mut Request, &'a mut Response, &'a mut Next), Output = &'a mut Response> + Send + Sync + 'static,
    {
        self.middlewares
            .push(register(Box::new(move |req, res, next| block_on(callback(req, res, next)))));

        return self;
    }

    pub(crate) fn is_match(&mut self, req: &'r mut Request) -> (bool, Values) {
        if self.method.to_uppercase() != req.method.to_uppercase() {
            return (false, Values::new());
        }

        return self.parameters_route_match(req);
    }

    fn dymanic_parameter_match(&mut self, path: String, matching: String) -> Option<(String, String)> {
        if PARAM_REGEX.is_match(&path) == false {
            return None;
        }

        return Some((path.trim_start_matches('{').trim_end_matches('}').to_owned(), matching));
    }
    
    // TODO: refactor
    fn parameters_route_match(&mut self, req: &'r mut Request) -> (bool, Values) {
        let mut parameters= Values::new();
        let route_path= uri_to_vec(self.path.clone());
        let request_path = uri_to_vec(req.path.clone());
        let domain_result = parse_host(format!("http://{}", req.host));

        if domain_result.is_none() {
            return (false, Values::new());
        }

        let subdomain_route: Vec<String> = self.subdomain.split(".").map(|v| v.to_string()).collect();
        let subdomain_req: Vec<String> = domain_result.unwrap().subdomain.split(".").map(|v| v.to_string()).collect();

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
