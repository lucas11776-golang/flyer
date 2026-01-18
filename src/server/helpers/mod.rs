use std::io::Result;

use crate::{
    request::Request,
    response::Response,
    server::protocol::http::APPLICATION,
    utils::cookie::cookie_parse
};


pub(crate) trait Handler {
    fn new() -> Self;
    async fn setup<'a>(&self, req: Request, res: Response) -> Result<(Request, Response)>;
    async fn teardown<'a>(&self, req: Request, res: Response) -> Result<(Request, Response)>;
}

pub(crate) struct RequestHandler;

impl Handler for RequestHandler {
    fn new() -> RequestHandler {
        return RequestHandler { };
    }

    #[allow(static_mut_refs)]
    async fn setup<'a>(&self, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        unsafe {
            if !req.is_asset() && APPLICATION.session_manager.is_some() {
                APPLICATION.session_manager
                    .as_mut()
                    .unwrap()
                    .setup(&mut req, &mut res)
                    .unwrap();
            }

            let cookie = cookie_parse(req.header("cookie"));

            if cookie.is_ok() {
                req.cookies.cookies = cookie.unwrap();
            }

            return Ok((req, res));
        }
    }

    #[allow(static_mut_refs)]
    async fn teardown<'a>(&self, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        unsafe {
            if res.view.is_some() && APPLICATION.view.is_some() {
                (req, res) = APPLICATION.view.as_mut().unwrap().render(req, res).unwrap();
            }

            if !req.is_asset() && APPLICATION.session_manager.is_some() {
                APPLICATION.session_manager
                    .as_mut()
                    .unwrap()
                    .teardown(&mut req, &mut res)
                    .unwrap();
            }

            return Ok((req, res));
        }
    }
}