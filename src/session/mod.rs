pub mod cookie;

use std::fmt::Debug;

use crate::request::Request;
use crate::response::Response;


// TODO: remember if sending struct to thread the must me thread safe...
pub trait Session: Send + Debug {
    fn set(&self, key: &str, value: &str);
    fn get(&self, key: &str) -> String; // Change to &self for object safety
}

pub trait SessionManager: Send + Sync {
    fn handle<'a>(&self, req: &'a mut Request, res: &'a mut Response) -> Box<dyn Session + 'a>;
}