use std::io::Result;

use crate::session::{Session, SessionManager};
use crate::response::Response;
use crate::request::Request;

pub fn new_session_cookie() -> impl SessionManager {
    return SessionCookieManager {

    }
}

pub struct SessionCookieManager {

}

pub(crate) struct SessionCookie {

}

impl SessionManager for SessionCookieManager {
    fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        // println!("COOKIE HANDLE");

        return Ok((req, res))
    }

    fn cleanup<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        // println!("COOKIE CLEANUP");

        return Ok((req, res))
    }

    fn expires(&mut self, expires_seconds: u128) {
        
    }
}


impl Session for SessionCookie {
    fn values(&mut self) -> crate::utils::Values {
        todo!()
    }

    fn set_values(&mut self, values: crate::utils::Values) -> crate::utils::Values {
        todo!()
    }

    fn get(&mut self, key: &str) -> String {
        todo!()
    }

    fn set(&mut self, key: &str, value: &str) {
        todo!()
    }

    fn errors(&mut self) -> crate::utils::Values {
        todo!()
    }

    fn set_errors(&mut self, errors: crate::utils::Values) -> crate::utils::Values {
        todo!()
    }

    fn set_error(&mut self, key: &str, value: &str) {
        todo!()
    }

    fn get_error(&mut self, key: &str) {
        todo!()
    }
}
