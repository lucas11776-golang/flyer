use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{
    request::Request, response::Response, routing::next::Next, utils::future::SendFuture
};

pub mod form;

#[allow(async_fn_in_trait)]
pub trait Hook: Send + Sync {
    async fn before(&self, req: Request, res: Response, next: Next) ->  Response;
    async fn after(&self, req: Request, res: Response, next: Next) -> Response;
}

pub trait HookErasure: Send + Sync {
    fn before(&self, req: Request, res: Response, next: Next) -> BoxFuture<'static, Response>;
    fn after(&self, req: Request, res: Response, next: Next) -> BoxFuture<'static, Response>;
}

pub struct HookWrapper<T: Hook + 'static> {
    pub instance: Arc<T>,
}

impl <T: Hook + 'static>HookWrapper<T> {
    pub fn new(instance: T) -> Self {
        return Self {
            instance: Arc::new(instance)
        };
    }
}

impl<T: Hook + 'static> HookErasure for HookWrapper<T> {
    fn before(&self, req: Request, res: Response, next: Next) -> BoxFuture<'static, Response> {
        let instance = Arc::clone(&self.instance);
        
        return Box::pin(async move {
            SendFuture(instance.before(req, res, next)).await
        });
    }

    fn after(&self, req: Request, res: Response, next: Next) -> BoxFuture<'static, Response> {
        let instance = Arc::clone(&self.instance);
        
        return Box::pin(async move {
            SendFuture(instance.after(req, res, next)).await
        });
    }
}