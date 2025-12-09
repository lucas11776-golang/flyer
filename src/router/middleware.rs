
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};

use lazy_static::lazy_static;

use crate::request::Request;
use crate::response::Response;
use crate::router::Middleware;
use crate::router::next::Next;


struct Container {
    middlewares:LazyLock<HashMap<String, Box<Middleware>>>
}  

impl Container {
    pub fn new() -> Self {
        return Self {
            middlewares: LazyLock::new(|| HashMap::new())
        }
    }

    pub fn insert(&mut self, call: Box<Middleware>) -> String {
        let reference = format!("{:p}", &call);

        self.middlewares.insert(reference.clone(), call);

        return reference;
    }

    pub fn call<'a>(&mut self, reference: String, req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
        return self.middlewares.get(&reference).unwrap().call((req, res, next));
    }
}


static mut VTABLE: LazyLock<RwLock<Container>> = LazyLock::new(|| RwLock::new(Container::new()));


#[allow(static_mut_refs)]
pub(crate) fn register(call: Box<Middleware>) -> String {
    return unsafe { VTABLE.get_mut().unwrap().insert(call) };
}

#[allow(static_mut_refs)]
pub(crate) fn call<'a>(reference: String, req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    return unsafe { VTABLE.get_mut().unwrap().call(reference, req, res, next) };
}
