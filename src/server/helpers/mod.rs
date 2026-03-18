use anyhow::Result;

use crate::{
    request::{Request, parser::parse_content_type},
    response::Response,
    server::Server,
    utils::cookie::cookie_parse
};

pub(crate) trait Handler {
    fn new() -> Self;
    async fn setup<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()>;
    async fn teardown<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()>;
}

pub(crate) struct RequestHandler { }

impl RequestHandler {
    fn setup_session<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        if req.is_asset() {
            return Ok(());
        }

        return Ok(match &mut Server::instance(ptr).session_manager  {
            Some(manager) =>  manager.setup(req, res).unwrap(),
            None => {},
        });
    }

    fn teardown_session<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        if req.is_asset() {
            return Ok(());
        }

        return Ok(match &mut Server::instance(ptr).session_manager {
            Some(manager) => manager.teardown(req, res).unwrap(),
            None => (),
        });
    }
}

impl Handler for RequestHandler {
    fn new() -> RequestHandler {
        return RequestHandler {};
    }

    async fn setup<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        if req.method == "POST" || req.method == "PATCH" || req.method == "PUT" {
            parse_content_type(req).await.unwrap();
        }

        if let Ok(cookies) = cookie_parse(req.header("cookie")) {
            req.cookies.cookies = cookies;
        }

        return self.setup_session(ptr, req, res);
    }

    async fn teardown<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        if res.view.is_some() {
            if let Some(view) = &mut Server::instance(ptr).view {
                view.render(req, res).unwrap();
            }
        }

        return self.teardown_session(ptr, req, res);
    }
}