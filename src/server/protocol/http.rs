use std::io::Result;
use std::sync::{LazyLock};
use std::thread::available_parallelism;

use rustls::ServerConfig;

use crate::assets::Assets;
use crate::request::Request;
use crate::response::Response;
use crate::router::group::GroupRouter;
use crate::server::helpers::{setup, teardown};
use crate::session::SessionManager;
use crate::utils::server::{TlsPathConfig, get_tls_config, server_config};
use crate::view::View;

pub(crate) type InitCallback = dyn Fn() + Send + Sync;

pub(crate) static mut APPLICATION: LazyLock<HTTP> = LazyLock::new(|| HTTP::new());

pub(crate) struct HTTP {
    pub(crate) host: String,
    pub(crate) port: i32,
    pub(crate) request_max_size: i64,
    pub(crate) router: GroupRouter,
    pub(crate) session_manager: Option<Box<dyn SessionManager>>,
    pub(crate) view: Option<View>,
    pub(crate) assets: Option<Assets>,
    pub(crate) parallelism_max_size: usize,
    pub(crate) server_config: Option<ServerConfig>,
    pub(crate) init_callback: Option<Box<InitCallback>>
}

impl HTTP {
    pub fn new() -> HTTP {
        return Self {
            host: String::from("127.0.0.1"),
            port: 80,
            request_max_size: 1024,
            router: GroupRouter::new(),
            view: None,
            session_manager: None,
            assets: None,
            parallelism_max_size: available_parallelism().unwrap().into(),
            server_config: None,
            init_callback: None,
        };
    }

    pub(crate) fn set_host(&mut self, host: &str) -> &mut Self {
        unsafe { APPLICATION.host = host.to_string(); }

        return self;
    }

    pub(crate) fn set_port(&mut self, port: i32) -> &mut Self {
        unsafe { APPLICATION.port = port; }

        return self;
    }

    pub(crate) fn set_tls(&mut self, tls_path: Option<TlsPathConfig>) -> &mut Self {
        if let Some(tls) = tls_path {
            unsafe { APPLICATION.server_config = Some(server_config(get_tls_config(&tls).unwrap()).unwrap()); }
        }
        
        return self;
    }

    #[allow(unused)]
    pub(crate) async fn on_request(&mut self, req: Request) -> Result<(Request, Response)> {
        let (mut req, mut res) = setup(req, Response::new()).await?;

        res.referer = req.header("referer");

        let result = self.router.web_match(&mut req, &mut res).await;

        if result.is_none() && self.assets.is_some() {
            (req, res) = self.assets.as_mut().unwrap().handle(req, res).unwrap();
        }

        return Ok(teardown(req, res).await.unwrap());
    }

    pub fn host(&self) -> String {
        return self.host.clone();
    }

    pub fn port(&self) -> i32 {
        return self.port;
    }

    pub fn address(&self) -> String {
        return std::format!("{0}:{1}", self.host, self.port);
    }
}



