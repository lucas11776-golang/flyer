
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::request::Request;
use crate::response::Response;
use crate::router::Middleware;
use crate::router::next::Next;


pub(crate) type Middlewares = HashMap<String, Box<Middleware>>;

lazy_static! {
    pub(crate) static ref MIDDLEWARES: Mutex<Middlewares> = Mutex::new(Middlewares::new());
}

pub(crate) fn register(middleware: Box<Middleware>) -> String {
    let pointer = format!("{:p}", &middleware);

    MIDDLEWARES.lock().unwrap().insert(pointer.clone(), middleware);

    return pointer;
}

pub(crate) fn call<'a>(pointer: String, req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    return MIDDLEWARES.lock().as_mut().unwrap().get(&pointer).unwrap()(req, res, next);
}
