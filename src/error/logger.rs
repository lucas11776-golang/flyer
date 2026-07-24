use std::{fmt, sync::Arc};

use futures::future::BoxFuture;

use crate::{
    request::Request,
    response::Response,
    utils::future::SendFuture
};

#[derive(Clone, Default, Debug)]
pub struct PanicErrorInfo {
    pub error: String,
    pub message: String,
}

impl PanicErrorInfo {
    pub fn new(error: String, message: String) -> Self {
        return Self {
            error: error,
            message,
        };
    } 
}

impl fmt::Display for PanicErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl std::error::Error for PanicErrorInfo {}

#[allow(async_fn_in_trait)]
pub trait Logger: Send + Sync {
    async fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> ();
}

pub(crate) trait LoggerErasure: Send + Sync {
    fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> BoxFuture<'static, ()>;
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
    fn call(&self, info: PanicErrorInfo, req: Request, res: Response) -> BoxFuture<'static, ()> {
        let instance = Arc::clone(&self.instance);

        return Box::pin(async move {
            SendFuture(instance.call(info, req, res)).await
        });
    }
}