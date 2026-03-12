use anyhow::Result;
use rustls::ServerConfig;
use tokio::{join, runtime::Builder};

use crate::{
    assets::Assets, request::Request, response::Response, router::{self, Router, routes::Routes}, server::{helpers::{Handler, RequestHandler}, transport::{tcp, udp}}, session::SessionManager, view::View
};

pub(crate) mod transport;
pub(crate) mod helpers;

use lazy_static::lazy_static;


lazy_static! {
    pub(crate) static ref SERVER: Option<Server> = None;
}



pub struct Server {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) routers: Vec<Box<Router>>,
    pub(crate) routes: Routes,
    pub(crate) session_manager: Option<Box<dyn SessionManager>>,
    pub(crate) view: Option<View>,
    pub(crate) assets: Option<Assets>,

    // pub(crate) parallelism_max_size: usize,

    pub(crate) server_config: Option<ServerConfig>,

}

impl Server {
    pub fn new(host: &str, port: u16, server_config: Option<ServerConfig>) -> Self {
        Self {
            host: String::from(host),
            port: port,
            routers: Vec::new(),
            routes: Routes::default(),
            session_manager: None,
            view: None,
            assets: None,

            // parallelism_max_size: parallelism_max_size,


            server_config: server_config
        }
    }

    pub fn address(&self) -> String {
        return format!("{}:{}", self.host, self.port);
    }

    pub fn router(&mut self) -> &mut Router {
        let idx = self.routers.len();

        self.routers.push(Box::new(Router {
            web: Vec::new(),
            ws: Vec::new(),
            path: String::new(),
            middlewares: Vec::new(),
            group: None,
            routers: Vec::new(),
            route_not_found_callback: None
        }));

        return self.routers[idx].as_mut();
    }

    pub fn assets(&mut self, path: &str, max_size_kilobytes_cache_size: usize, expires_in_seconds: u128) -> &mut Self {
        self.assets = Some(Assets::new(path.to_owned(), max_size_kilobytes_cache_size, expires_in_seconds));

        return self;
    }

    pub fn session(&mut self, manager: impl SessionManager + 'static) -> &mut Self {
        self.session_manager = Some(Box::new(manager));

        return self;
    }

    pub fn view(&mut self, path: &str) -> &mut Self {
        self.view = Some(View::new(path));

        return self;
    }

    // TODO: this must handle web and ws requests
    pub(crate) async fn on_request<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        let handler = RequestHandler::new();

        handler.setup(req, res).await?;
        res.referer = req.header("referer");
        self.routes.handle_web_request(req, res);

        if self.assets.is_some() {
            self.assets.as_mut().unwrap().handle(req, res).unwrap();
        }

        return Ok(handler.teardown(req, res).await.unwrap());
    }

    pub(crate) async fn init(&mut self) {
        router::resolver::resolve(self);
        // Using memory address to avoid compiler checks we server will not be mut
        // Getting server value - (*(server_ptr as *const &mut Server))
        join!(tcp::listen(&self as *const &mut Self as usize), udp::listen(&self as *const &mut Self as usize));
    }

    pub fn listen(&mut self) {
        Builder::new_multi_thread()
            .worker_threads(8)
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.init());
    }

}
