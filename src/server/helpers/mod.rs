use std::io::Result;

use crate::{
    request::Request,
    response::Response,
    server::protocol::http::GLOBAL_HTTP,
    utils::cookie::cookie_parse
};


// TODO: refactor this
#[allow(static_mut_refs)]
pub(crate) async fn setup<'a>(mut req: Request, mut res: Response) -> Result<(Request, Response)> {
    unsafe {
        if !req.is_asset() && GLOBAL_HTTP.session_manager.is_some() {
            GLOBAL_HTTP.session_manager
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

// TODO: refactor this
#[allow(static_mut_refs)]
pub(crate) async fn teardown<'a>(mut req: Request, mut res: Response) -> Result<(Request, Response)> {
    unsafe {
        if res.view.is_some() && GLOBAL_HTTP.view.is_some() {
            (req, res) = GLOBAL_HTTP.view.as_mut().unwrap().render(req, res).unwrap();
        }

        if !req.is_asset() && GLOBAL_HTTP.session_manager.is_some() {
            GLOBAL_HTTP.session_manager
                .as_mut()
                .unwrap()
                .teardown(&mut req, &mut res)
                .unwrap();
        }

        return Ok((req, res));
    }
}