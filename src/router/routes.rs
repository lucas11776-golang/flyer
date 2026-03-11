use anyhow::Result;
use reqwest::Response;

use crate::{request::Request, router::{Route, WebRoute, WsRoute}};

#[derive(Debug, Default)]
pub(crate) struct Routes {
    pub(crate) web: Vec<Box<Route<WebRoute>>>,
    pub(crate) ws: Vec<Box<Route<WsRoute>>>,
}

impl Routes {
    pub(crate) async fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        println!("HANDLING REQUESt");
        return Ok((req, res));
    }
}
