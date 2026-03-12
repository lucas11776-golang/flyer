use std::collections::HashMap;
use std::sync::LazyLock;

use crate::request::Request;
use crate::response::Response;
use crate::router::Middleware;
use crate::router::next::Next;

static mut CONTAINER: LazyLock<Container> = LazyLock::new(|| Container::new());

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
        let fat_ptr: *const Middleware = Box::into_raw(call);
        let [_, vtable_ptr] = unsafe { std::mem::transmute_copy::<*const Middleware, [usize; 2]>(&fat_ptr) };
        let call = unsafe { Box::from_raw(fat_ptr as *mut Middleware) };
        let ptr = format!("0x{:x}", vtable_ptr);

        self.middlewares.insert(ptr.clone(), call);
        
        return ptr;
    }

    pub fn call<'a>(&mut self, reference: String, req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
        return self.middlewares.get(&reference).unwrap()(req, res, next);
    }
}

#[allow(static_mut_refs)]
pub(crate) fn register(call: Box<Middleware>) -> String {
    return unsafe { CONTAINER.insert(call) };
}

#[allow(static_mut_refs)]
pub(crate) fn call<'a>(reference: String, req: &'a mut Request, res: &'a mut Response, next: &'a mut Next) -> &'a mut Response {
    return unsafe { CONTAINER.call(reference, req, res, next) };
}
