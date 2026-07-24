// use async_trait::async_trait;
// use crate::{request::Request, response::Response};

// #[async_trait]
// pub trait Hook: Send + Sync {
//     async fn before(&mut self, req: Request, res: Response) -> (Request, Response);
//     async fn after(&mut self, req: Request, res: Response) -> (Request, Response);
// }