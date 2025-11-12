pub mod cookie;

use std::io::Result;

use crate::request::Request;
use crate::response::Response;
use crate::utils::Values;

pub trait Session: Send + Sync {
    fn values(&mut self) -> Values;
    fn set(&mut self, key: &str, value: &str);
    fn set_values(&mut self, values: Values);
    fn get(&mut self, key: &str) -> String;
    fn remove(&mut self, key: &str);
    fn errors(&mut self) -> Values;
    fn set_error(&mut self, key: &str, value: &str);
    fn set_errors(&mut self, errors: Values);
    fn get_error(&mut self, key: &str) -> String;
    fn remove_error(&mut self, key: &str);
}

pub trait SessionManager: Send + Sync {
    fn setup<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)>;
    fn teardown<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)>;
}