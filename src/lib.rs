use once_cell::sync::OnceCell;
use rustls::ServerConfig;

use crate::{
    server::Server,
    utils::server::{get_tls_config, server_config, TlsPathConfig},
};

pub mod cookies;
pub mod error;
pub mod hooks;
pub mod loggers;
pub mod mail;
pub mod request;
pub mod response;
pub mod routing;
pub mod server;
pub mod session;
pub mod storage;
pub mod types;
pub mod utils;
pub mod validation;
pub mod view;
pub mod websocket;

pub use anyhow::{self, Result};
pub use request::Request;
pub use response::Response;
pub use websocket::Websocket;

pub(crate) static mut GLOBAL_SERVER: OnceCell<Server> = OnceCell::new();

#[allow(static_mut_refs)]
#[inline]
fn init_global_server<'s>(
    host: impl Into<String>,
    port: u32,
    config: Option<ServerConfig>,
) -> &'s mut Server {
    unsafe {
        GLOBAL_SERVER
            .set(Server::new(host.into(), port, config))
            .map_err(|_| "global state already initialized")
            .unwrap();

        GLOBAL_SERVER.get_mut().unwrap()
    }
}

#[allow(static_mut_refs)]
pub fn server<'s>(host: impl Into<String>, port: u32) -> &'s mut Server {
    init_global_server(host, port, None)
}

#[allow(static_mut_refs)]
pub fn server_tls<'s>(
    host: impl Into<String>,
    port: u32,
    key_path: impl AsRef<str>,
    cert_path: impl AsRef<str>,
) -> &'s mut Server {
    let tls_path = TlsPathConfig::new(key_path.as_ref(), cert_path.as_ref());
    let tls_config = get_tls_config(&tls_path).unwrap();
    let config = Some(server_config(tls_config).unwrap());

    init_global_server(host, port, config)
}