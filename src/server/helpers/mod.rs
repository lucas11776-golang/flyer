use std::io::Result;

use crate::{
    GLOBAL_SERVER, request::Request, response::Response, utils::cookie::cookie_parse
};


pub(crate) trait Handler {
    fn new() -> Self;
    async fn setup<'a>(&self, req: &'a mut Request, res: &'a mut Response) -> Result<()>;
    async fn teardown<'a>(&self, req: &'a mut Request, res: &'a mut Response) -> Result<()>;
}

// TODO: Give it better name and refactor
pub(crate) struct RequestHandler;

impl Handler for RequestHandler {
    fn new() -> RequestHandler {
        return RequestHandler { };
    }

    #[allow(static_mut_refs)]
    async fn setup<'a>(&self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        unsafe {
            let server = GLOBAL_SERVER.get_mut().unwrap();

            if !req.is_asset() && server.session_manager.is_some() {
                server.session_manager
                    .as_mut()
                    .unwrap()
                    .setup(req, res)
                    .unwrap();
            }

            let cookie = cookie_parse(req.header("cookie"));

            if cookie.is_ok() {
                req.cookies.cookies = cookie.unwrap();
            }

            return Ok(());
        }
    }

    #[allow(static_mut_refs)]
    async fn teardown<'a>(&self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        unsafe {
            let server = GLOBAL_SERVER.get_mut().unwrap();

            // println!("SOME -> {} -- {}", res.view.is_some(), server.view.is_some());

            if res.view.is_some() && server.view.is_some() {
                server.view.as_mut().unwrap().render(req, res).unwrap();
            }

            if !req.is_asset() && GLOBAL_SERVER.get_mut().unwrap().session_manager.is_some() {
                GLOBAL_SERVER.get_mut().unwrap().session_manager
                    .as_mut()
                    .unwrap()
                    .teardown(req, res)
                    .unwrap();
            }

            return Ok(());
        }
    }
}