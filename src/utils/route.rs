use std::collections::HashSet;

use crate::{routing::MiddlewareHandler, server::Server, utils::mem::Instance};

pub fn get_middleware_pointer(callback: MiddlewareHandler) -> (String, Box<MiddlewareHandler>) {
    let call: MiddlewareHandler = Box::new(callback);
    let raw_ptr: *mut MiddlewareHandler = Box::into_raw(Box::new(call));
    let ptr = format!("{:p}", raw_ptr);
    let re_call: Box<MiddlewareHandler> = unsafe { Box::from_raw(raw_ptr) };

    return (ptr, re_call);
}

pub fn middleware_resolver(server: Instance<Server>, callback: MiddlewareHandler, middlewares: &mut HashSet<String>) {
    let (k, v) = get_middleware_pointer(callback);

    server.as_mut().routes.middlewares.insert(k.clone(), v);

    middlewares.insert(k);   
}