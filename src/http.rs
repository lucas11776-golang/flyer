use std::sync::{LazyLock};
use std::thread::available_parallelism;

use crate::assets::Assets;
use crate::router::group::GroupRouter;
use crate::session::SessionManager;
use crate::utils::server::TlsPathConfig;
use crate::view::View;

pub(crate) static mut HTTP_CONTAINER: LazyLock<HTTP> = LazyLock::new(|| HTTP::new());

pub(crate) struct HTTP {
    pub(crate) host: String,
    pub(crate) port: i32,
    pub(crate) tls: Option<TlsPathConfig>,
    pub(crate) request_max_size: i64,
    pub(crate) router: GroupRouter,
    pub(crate) session_manager: Option<Box<dyn SessionManager>>,
    pub(crate) view: Option<View>,
    pub(crate) assets: Option<Assets>,
    pub(crate) parallelism_max_size: usize,
}

impl HTTP {
    pub fn new() -> HTTP {
        return Self {
            host: String::from("127.0.0.1"),
            port: 80,
            tls: None,
            request_max_size: 1024,
            router: GroupRouter::new(),
            view: None,
            session_manager: None,
            assets: None,
            parallelism_max_size: available_parallelism().unwrap().into(),
        };
    }

    pub(crate) fn set_host(&mut self, host: &str) -> &mut Self {
        unsafe { HTTP_CONTAINER.host = host.to_string(); }

        return self;
    }

    pub(crate) fn set_port(&mut self, port: i32) -> &mut Self {
        unsafe { HTTP_CONTAINER.port = port; }

        return self;
    }

    pub(crate) fn set_tls(&mut self, tls: Option<TlsPathConfig>) -> &mut Self {
        unsafe { HTTP_CONTAINER.tls = tls; }

        return self;
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