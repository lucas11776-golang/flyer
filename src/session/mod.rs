pub mod cookie;

use std::io::Result;

use crate::request::Request;
use crate::response::Response;
use crate::utils::Values;

pub trait Session: Send + Sync {
    fn values(&mut self) -> Values;
    fn set_values(&mut self, values: Values) -> Values;
    fn get(&mut self, key: &str) -> String;
    fn set(&mut self, key: &str, value: &str);
    fn errors(&mut self) -> Values;
    fn set_errors(&mut self, errors: Values) -> Values;
    fn set_error(&mut self, key: &str, value: &str);
    fn get_error(&mut self, key: &str);
}

pub trait SessionManager: Send + Sync {
    fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)>;
    fn cleanup<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)>;
    fn expires(&mut self, expires_seconds: u128); 
}