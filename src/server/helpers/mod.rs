use std::io::Result;

use crate::{
    HTTP,
    request::Request,
    response::Response, utils::cookie::cookie_parse
};

pub(crate) async fn setup<'a>(http: &'a mut HTTP, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
    if !req.is_asset() && http.session_manager.is_some() {
        http.session_manager
            .as_mut()
            .unwrap()
            .setup(&mut req, &mut res)
            .unwrap();
    }

    let cookie = cookie_parse(req.header("cookie"));

    if cookie.is_ok() {
        req.cookies.cookies = cookie.unwrap();
    }

    return Ok((req, res))
}

pub(crate) async fn teardown<'a>(http: &'a mut HTTP, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
    if res.view.is_some() && http.view.is_some() {
        (req, res) = http.view.as_mut().unwrap().render(req, res).unwrap();
    }


    if !req.is_asset() && http.session_manager.is_some() {
        http.session_manager
            .as_mut()
            .unwrap()
            .teardown(&mut req, &mut res)
            .unwrap();
    }

    return Ok((req, res))
}