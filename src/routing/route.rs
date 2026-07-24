use std::collections::HashSet;

use crate::{
    request::Request,
    response::Response,
    routing::next::Next,
    server::Server,
    utils::{mem::Instance, route::middleware_resolver}
};

pub struct Route<H> {
    pub(crate) server: Instance<Server>,
    pub(crate) method: String,
    pub(crate) subdomain: String,
    pub(crate) path: Vec<String>,
    pub(crate) handler: H,
    pub(crate) middlewares: HashSet<String>
}

impl <H>Route<H> {
    pub fn middleware<C, Fut>(&mut self, callback: C) -> &mut Self
    where
        C: Fn(Request, Response, Next) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        middleware_resolver(
            self.server.clone(),
            Box::new(move |req, res, next| Box::pin(callback(req, res, next))),
            &mut self.middlewares
        );

        return self;
    }
}