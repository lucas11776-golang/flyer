use once_cell::sync::OnceCell;

use crate::{
    server::Server,
    utils::server::{TlsPathConfig, get_tls_config, server_config}
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

pub(crate) static mut GLOBAL_SERVER: OnceCell<Box<Server>> = OnceCell::new();

#[allow(static_mut_refs)]
pub fn server<'s>(host: impl Into<String>, port: u32) -> &'s mut Server {
    return unsafe {
        let server = Server::new(host.into(), port, None);

        GLOBAL_SERVER
            .set(Box::new(server))
            .map_err(|_| "global state already initialized")
            .unwrap();

        GLOBAL_SERVER.get_mut().unwrap().as_mut()
    }
}

#[allow(static_mut_refs)]
pub fn server_tls<'s>(host: impl Into<String>, port: u32, key_path: impl Into<String>, cert_path: impl Into<String>) -> &'s mut Server {
    return unsafe {
        let server_config = Some(server_config(get_tls_config(&TlsPathConfig::new(&key_path.into(), &cert_path.into())).unwrap()).unwrap());
        let server = Server::new(host.into(), port, server_config);

        GLOBAL_SERVER
            .set(Box::new(server))
            .map_err(|_| "global state already initialized")
            .unwrap();

        GLOBAL_SERVER.get_mut().unwrap().as_mut()
    }
}