use std::thread::available_parallelism;

use anyhow::Result;
use async_std::task::block_on;
use rustls::ServerConfig;
use tokio::{join, runtime::Builder};

use crate::{
    assets::Assets,
    request::Request,
    response::Response,
    router::{Router, WsRoute, resolver::RouterResolver, route::Route, routes::Routes},
    server::{helpers::{Handler, RequestHandler},
    transport::{tcp, udp}},
    session::{SessionManager, file::FileSessionManager},
    view::View
};

pub(crate) mod transport;
pub(crate) mod helpers;

pub(crate) type InitCallback = dyn Fn() + Send + Sync;

pub struct Server {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) routers: Vec<Box<Router>>,
    pub(crate) routes: Routes,
    pub(crate) session_manager: Option<Box<dyn SessionManager>>,
    pub(crate) view: Option<View>,
    pub(crate) assets: Option<Assets>,
    pub(crate) request_max_size: usize,
    pub(crate) parallelism_max_size: usize,
    pub(crate) server_config: Option<ServerConfig>,
    pub(crate) init_callback: Option<Box<InitCallback>>,
}

impl Server {
    pub fn new(host: &str, port: u16, server_config: Option<ServerConfig>) -> Self {
        Self {
            host: String::from(host),
            port: port,
            routers: Vec::new(),
            routes: Routes::default(),
            view: None,
            assets: None,
            request_max_size: (1024 * 20) * 1000,
            parallelism_max_size: available_parallelism().unwrap().into(),
            server_config: server_config,
            init_callback: None,
            session_manager: Some(Box::new(FileSessionManager::new(None))),
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
            subdomain: Vec::new(),
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


    pub fn set_request_max_size(&mut self, kilobytes: usize) -> &mut Self {
        self.request_max_size = kilobytes * 1000;

        return self;
    }

    pub fn set_max_parallelism(&mut self, cores: usize) -> &mut Self {
        self.parallelism_max_size = cores;

        return self;
    }

    pub fn init<C>(&mut self, callback: C)
    where
        C: AsyncFn() -> () + Send + Sync + 'static
    {
        self.init_callback = Some(Box::new(move || block_on(callback())));
    }

    pub fn listen(&mut self) {
        Builder::new_multi_thread()
            .worker_threads(self.parallelism_max_size)
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.start_server());
    }

    async fn start_server(&mut self) {
        RouterResolver::resolve(self);
        // Using memory address to avoid compiler checks we server will not be mut
        // Getting server value - (*(server_ptr as *const &mut Server))
        let ptr = &self as *const &mut Self as usize;

        if self.init_callback.is_some() {
            tokio::spawn(async move {
                Self::instance(ptr).init_callback.as_mut().unwrap()();
            });
        }

        join!(tcp::listen(ptr), udp::listen(ptr));
    }

    pub(crate) async fn on_web_request<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        let handler = RequestHandler::new();
        let ptr = &self as *const &mut Self as usize;

        handler.setup(ptr, req, res).await.unwrap();
        res.referer = req.header("referer");
        self.routes.handle_web_request(req, res);

        // Route not found check if path exist in assets.
        if res.status_code == 404 {
            if let Some(assets) = &mut self.assets {
                assets.handle(req, res);
            }
        }

        return handler.teardown(ptr, req, res).await;
    }

    pub(crate) async fn on_ws_request<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Option<(&'a Route<WsRoute>, &'a mut Request, &'a mut Response)> {
        let ptr = &self as *const &mut Self as usize;

        RequestHandler::new().setup(ptr, req, res).await.unwrap();

        return self.routes.handle_ws_request(req, res);
    }

    pub(crate) fn instance<'s>(ptr: usize) -> &'s mut Self {
        return unsafe { *(ptr as *mut &mut Server) };
    }
}
