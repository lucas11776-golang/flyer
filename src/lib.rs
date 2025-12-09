#![feature(async_fn_traits)]
#![feature(unboxed_closures)]
#![feature(downcast_unchecked)]
#![feature(type_alias_impl_trait)]
#![feature(fn_traits)]

pub mod http;
pub mod request;
pub mod response;
pub mod ws;
pub mod router;
pub mod utils;
pub mod session;
pub mod view;
pub mod server;
pub mod assets;
pub mod cookie;
pub mod validation;

use rustls::ServerConfig;
use tokio::{join, runtime::Runtime};

use crate::{
    assets::Assets,
    http::HTTP_CONTAINER,
    router::Router,
    server::{TlsPathConfig, get_tls_config, server_config, tcp::TcpServer, udp::UdpServer},
    session::SessionManager,
    utils::load_env, view::View
};

pub struct Server;

impl Server {
    #[allow(static_mut_refs)]
    pub fn new(host: &str, port: i32, tls: Option<TlsPathConfig>) -> Self {
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

            Runtime::new().unwrap().block_on(async {
                join!(
                    Self::udp_server(config.clone()),
                    Self::tcp_server(config),
                );
            });
        }
    }

    async fn tcp_server(config: Option<ServerConfig>) {
        TcpServer::new(config)
            .await
            .unwrap()
            .listen()
            .await;
    }

    async fn udp_server(config: Option<ServerConfig>) {
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


#[allow(static_mut_refs)]
pub fn server<'a>(host: &str, port: i32) -> Server {
    return Server::new(host, port, None);
}

#[allow(static_mut_refs)]
pub fn server_tls<'a>(host: &str, port: i32, key: &str, cert: &str) -> Server {
    return Server::new(host, port, Some(TlsPathConfig {
        key_path: key.to_owned(),
        cert_path: cert.to_owned()
    }));
}