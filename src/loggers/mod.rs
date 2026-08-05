use std::sync::Arc;

use futures::future::BoxFuture;

use crate::{
    error::Error,
    request::Request,
    response::Response,
    utils::future::SendFuture
};

pub mod sentry;

#[allow(async_fn_in_trait)]
pub trait Logger: Send + Sync {
    async fn call(&self, info: Error, req: Request, res: Response) -> ();
}

pub(crate) trait LoggerErasure: Send + Sync {
    fn call(&self, info: Error, req: Request, res: Response) -> BoxFuture<'static, ()>;
}

pub struct LoggerWrapper<T: Logger + 'static> {
    pub instance: Arc<T>,
}

impl <T: Logger + 'static>LoggerWrapper<T> {
    pub fn new(instance: T) -> Self {
        return Self {
            instance: Arc::new(instance)
        };
    }
}

impl<T: Logger + 'static> LoggerErasure for LoggerWrapper<T> {
    fn call(&self, info: Error, req: Request, res: Response) -> BoxFuture<'static, ()> {
        let instance = Arc::clone(&self.instance);

        return Box::pin(async move {
            SendFuture(instance.call(info, req, res)).await
        });
    }
}