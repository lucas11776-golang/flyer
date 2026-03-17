use anyhow::Result;

use crate::{request::{Request, parser::parse_content_type}, response::Response, server::Server, utils::cookie::cookie_parse};

pub(crate) trait Handler {
    fn new() -> Self;
    async fn setup<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()>;
    async fn teardown<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()>;
}

pub(crate) struct RequestHandler;

impl RequestHandler {
    fn handle_session<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        if !req.is_asset() {
            if let Some(manager) = &mut Server::instance(ptr).session_manager {
                manager.setup(req, res).unwrap();
            }
        }

        return Ok(());
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

        return self.handle_session(ptr, req, res);
    }

    async fn teardown<'a>(&self, ptr: usize, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        if res.view.is_some() {
            if let Some(view) = &mut Server::instance(ptr).view {
                view.render(req, res).unwrap();
            }
        }

        return self.handle_session(ptr, req, res);
    }
}