pub mod udp;
pub mod tcp;
pub mod handler;
pub mod helpers;


use rustls::ServerConfig;
use tokio::{join, runtime::Builder};

use crate::{
    assets::Assets,
    http::HTTP_CONTAINER,
    router::Router,
    server::{
        udp::UdpServer
    },
    session::SessionManager,
    utils::{
        load_env,
        server::{TlsPathConfig, get_tls_config, server_config}
    },
    view::View
};

pub enum Protocol {
    HTTP1,
    HTTP2,
    HTTP3
}

pub struct Server;

impl Server {
    #[allow(static_mut_refs)]
    pub(crate) fn new(host: &str, port: i32, tls: Option<TlsPathConfig>) -> Self {
        unsafe { HTTP_CONTAINER.set_host(host).set_port(port).set_tls(tls) };

        return Self {}
    }

    pub fn env(self, path: &str) -> Self {
        load_env(path);

        return self;
    }

    #[allow(static_mut_refs)]
    pub fn host(&self) -> String {
        return unsafe { HTTP_CONTAINER.host() };
    }

    #[allow(static_mut_refs)]
    pub fn port(&self) -> i32 {
        return unsafe { HTTP_CONTAINER.port() };
    }

    #[allow(static_mut_refs)]
    pub fn address(&self) -> String {
        return unsafe { HTTP_CONTAINER.address()};
    }

    pub fn set_request_max_size(self, size: i64) -> Self {
        unsafe { HTTP_CONTAINER.request_max_size = size; }

        return self;
    }

    pub fn set_max_parallelism(self, number: usize) -> Self {
        unsafe { HTTP_CONTAINER.parallelism_max_size = number; }

        return self;
    }

    pub fn view(self, path: &str) -> Self {
        unsafe { HTTP_CONTAINER.view = Some(View::new(path)); }

        return self;
    }

    pub fn assets(self, path: &str, max_size_kilobytes_cache_size: usize, expires_in_seconds: u128) -> Self {
        unsafe { HTTP_CONTAINER.assets = Some(Assets::new(path.to_owned(), max_size_kilobytes_cache_size, expires_in_seconds)); }

        return self;
    }

    pub fn session(self, manager: impl SessionManager + 'static) -> Self {
        unsafe { HTTP_CONTAINER.session_manager = Some(Box::new(manager)); }

        return self;
    }

    #[allow(static_mut_refs)]
    pub fn router<'a>(&mut self) -> &mut Router {
        unsafe { 
            let idx = HTTP_CONTAINER.router.nodes.len();

            HTTP_CONTAINER.router.nodes.push(Box::new(Router::new()));

            return &mut HTTP_CONTAINER.router.nodes[idx];
        }
    }

    #[allow(static_mut_refs)]
    pub fn listen(&mut self) {
        unsafe { 
            HTTP_CONTAINER.router.init();

            let mut config: Option<ServerConfig> = None;

            if HTTP_CONTAINER.tls.is_some() {
                config = Some(server_config(get_tls_config(&HTTP_CONTAINER.tls.as_mut().unwrap()).unwrap()).unwrap());
            }

            Builder::new_multi_thread()
                .worker_threads(HTTP_CONTAINER.parallelism_max_size)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { join!(Self::udp(config.clone()), tcp::listen(config)); });
        }
    }

    async fn udp(config: Option<ServerConfig>) {
        if config.is_none() {
            return;
        }

        UdpServer::new(config.unwrap())
            .await
            .unwrap()
            .listen()
            .await;
    }
}
