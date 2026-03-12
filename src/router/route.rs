use async_std::task::block_on;
use regex::Regex;
use url_domain_parse::Url;
use once_cell::sync::Lazy;

use crate::{
    request::Request,
    response::Response,
    router::{middleware::register, next::Next},
    utils::{Values, url::uri_to_vec}
};

static PARAM_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{[a-zA-Z_]+\}").expect("Invalid parameter regex"));

pub struct Route<Handler: ?Sized> {
    pub(crate) subdomain: String,
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

    fn dynamic_parameter_match(&self, path: String, matching: String) -> Option<(String, String)> {
        if PARAM_REGEX.is_match(&path) == false {
            return None;
        }

        return Some((path.trim_start_matches('{').trim_end_matches('}').to_owned(), matching));
    }
    
    // TODO: refactor
    fn parameters_route_match<'a>(&self, req: &'a mut Request) -> (bool, Values) {
        let mut parameters= Values::new();
        let route_path= uri_to_vec(String::from(self.path.trim_matches('/')));
        let request_path = uri_to_vec(req.path.clone());
        // TODO: fix plugin Domain parser.
        let url_result = Url::parse(format!("http://{}", &req.host).as_str()); // TODO: fix

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

                if let Some((k, v)) = self.dynamic_parameter_match(subdomain_route[j].clone(), subdomain_req[j].clone()) {                    
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

            if let Some((k, v)) = self.dynamic_parameter_match(route_path[i].clone(), request_path[i].clone()) {
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
