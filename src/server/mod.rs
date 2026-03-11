use anyhow::Result;
use rustls::ServerConfig;
use tokio::{join, runtime::Builder};

use crate::{
    request::Request,
    response::Response,
    router::{self, Router, routes::Routes}, server::transport::{tcp, udp}
};

pub(crate) mod transport;

pub struct Server {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) routers: Vec<Box<Router>>,
    pub(crate) routes: Routes,
    pub(crate) server_config: Option<ServerConfig>,

}

impl Server {
    pub fn new(host: &str, port: u16, server_config: Option<ServerConfig>) -> Self {
        Self {
            host: String::from(host),
            port: port,
            routers: Vec::new(),
            routes: Routes::default(),
            server_config: server_config
        }
    }

    pub fn address(&self) -> String {
        return format!("{}:{}", self.host, self.port);
    }

    pub fn router(&mut self) -> &mut Router {
        let idx = self.routers.len();

        self.routers.push(Box::new(Router {
            web_routes: Vec::new(),
            ws_routes: Vec::new(),
            path: String::new(),
            middlewares: Vec::new(),
            group: None,
            routers: Vec::new(),
        }));

        return self.routers[idx].as_mut();
    }

    pub(crate) fn on_request<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        return Ok((req, res))
    }

    pub(crate) async fn init(mut self) {
        router::resolver::resolve(&mut self);
        
        join!(tcp::listen(&self as *const Self as usize), udp::listen(&self as *const Self as usize));
    }

    pub fn listen(self) {
        Builder::new_multi_thread()
            .worker_threads(8)
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.init());
    }

}
