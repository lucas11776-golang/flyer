pub(crate) mod handler;
pub(crate) mod helpers;
pub(crate) mod transport;
pub(crate) mod protocol;

use tokio::{join, runtime::Builder};

use crate::{
    assets::Assets,
    router::Router,
    server::{protocol::http::GLOBAL_HTTP, transport::{tcp, udp}},
    session::SessionManager,
    utils::{load_env, server::TlsPathConfig},
    view::View
};

pub struct Server;

impl Server {
    #[allow(static_mut_refs)]
    pub(crate) fn new(host: &str, port: i32, tls: Option<TlsPathConfig>) -> Self {
        unsafe { GLOBAL_HTTP.set_host(host).set_port(port).set_tls(tls) };

        return Self {}
    }

    pub fn env(self, path: &str) -> Self {
        load_env(path);

        return self;
    }

    #[allow(static_mut_refs)]
    pub fn host(&self) -> String {
        return unsafe { GLOBAL_HTTP.host() };
    }

    #[allow(static_mut_refs)]
    pub fn port(&self) -> i32 {
        return unsafe { GLOBAL_HTTP.port() };
    }

    #[allow(static_mut_refs)]
    pub fn address(&self) -> String {
        return unsafe { GLOBAL_HTTP.address()};
    }

    pub fn set_request_max_size(self, size: i64) -> Self {
        unsafe { GLOBAL_HTTP.request_max_size = size; }

        return self;
    }

    pub fn set_max_parallelism(self, number: usize) -> Self {
        unsafe { GLOBAL_HTTP.parallelism_max_size = number; }

        return self;
    }

    pub fn view(self, path: &str) -> Self {
        unsafe { GLOBAL_HTTP.view = Some(View::new(path)); }

        return self;
    }

    pub fn assets(self, path: &str, max_size_kilobytes_cache_size: usize, expires_in_seconds: u128) -> Self {
        unsafe { GLOBAL_HTTP.assets = Some(Assets::new(path.to_owned(), max_size_kilobytes_cache_size, expires_in_seconds)); }

        return self;
    }

    pub fn session(self, manager: impl SessionManager + 'static) -> Self {
        unsafe { GLOBAL_HTTP.session_manager = Some(Box::new(manager)); }

        return self;
    }

    #[allow(static_mut_refs)]
    pub fn router<'a>(&mut self) -> &mut Router {
        unsafe { 
            let idx = GLOBAL_HTTP.router.nodes.len();

            GLOBAL_HTTP.router.nodes.push(Box::new(Router::new()));

            return &mut GLOBAL_HTTP.router.nodes[idx];
        }
    }

    #[allow(static_mut_refs)]
    pub fn listen(self) {
        unsafe { 
            GLOBAL_HTTP.router.init();

            Builder::new_multi_thread()
                .worker_threads(GLOBAL_HTTP.parallelism_max_size)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { join!(tcp::listen(), udp::listen()) });
        }
    }
}
