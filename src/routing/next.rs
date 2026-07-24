use std::mem;

use crate::{
    request::Request,
    response::Response,
};

#[derive(Clone)]
pub struct Next {
    inner: Option<Request>,
}

impl Next {
    pub(crate) fn new() -> Self {
        return Self {
            inner: None
        };
    }

    pub fn handle(mut self, req: Request, mut res: Response) -> Response {
        self.inner = Some(req);
        res.next = Some(self);
        res.next(true);
        return res;
    }

    pub(crate) fn request(&mut self) -> Request {
        return mem::take(&mut self.inner).unwrap();
    }
}