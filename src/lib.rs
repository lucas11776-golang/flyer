use once_cell::sync::OnceCell;

use crate::{
    server::{Server},
    utils::server::{TlsPathConfig, get_tls_config, server_config}
};

pub mod router;
pub mod request;
pub mod response;
pub mod ws;
pub mod server;
pub mod utils;
pub mod cookie;
pub mod session;
pub mod view;
pub mod validation;
pub mod assets;

static mut GLOBAL_SERVER: OnceCell<Box<Server>> = OnceCell::new();

#[allow(static_mut_refs)]
pub fn server<'a>(host: &str, port: u16) -> &'a mut Server {
    return unsafe {
        let server = Server::new(host, port, None);

        GLOBAL_SERVER
            .set(Box::new(server))
            .map_err(|_| "global state already initialized")
            .unwrap();

        GLOBAL_SERVER.get_mut().unwrap().as_mut()
    };
}

#[allow(static_mut_refs)]
pub fn server_tls<'a>(host: &str, port: u16, key_path: &str, cert_path: &str) -> &'a mut Server {
    return unsafe {
        let server = Server::new(
            host,
            port,
            Some(server_config(get_tls_config(&TlsPathConfig::new(key_path, cert_path)).unwrap()).unwrap())
        );

        GLOBAL_SERVER
            .set(Box::new(server))
            .map_err(|_| "global state already initialized")
            .unwrap();

        GLOBAL_SERVER.get_mut().unwrap().as_mut()
    };
}

#[macro_export]
macro_rules! info {
    ($msg:expr $(; $($k:expr => $v:expr),* )? ) => {{
        let logger = crate::utils::logger::logger();
        slog::info!(logger, $msg; "level" => "info" $(, $($k => $v),* )? );
    }};
    ($msg:expr, $($arg:tt)+) => {{
        let logger = crate::utils::logger::logger();
        slog::info!(logger, $msg, $($arg)+);
    }};
}

#[macro_export]
macro_rules! warn {
    ($msg:expr $(; $($k:expr => $v:expr),* )? ) => {{
        let logger = crate::utils::logger::logger();
        slog::warn!(logger, $msg; "level" => "warn" $(, $($k => format!("{}", $v)),* )? );
    }};
    ($msg:expr, $($arg:tt)+) => {{
        let logger = crate::utils::logger::logger();
        slog::warn!(logger, $msg, $($arg)+);
    }};
}

#[macro_export]
macro_rules! success {
    ($msg:expr $(; $($k:expr => $v:expr),* )? ) => {{
        let logger = crate::utils::logger::logger();
        slog::info!(logger, $msg; "level" => "success" $(, $($k => $v),* )? );
    }};
    ($msg:expr, $($arg:tt)+) => {{
        let logger = crate::utils::logger::logger();
        slog::info!(logger, $msg, $($arg)+);
    }};
}

#[macro_export]
macro_rules! danger {
    ($msg:expr $(; $($k:expr => $v:expr),* )? ) => {{
        let logger = crate::utils::logger::logger();
        slog::error!(logger, $msg; "level" => "danger" $(, $($k => $v),* )? );
    }};
    ($msg:expr, $($arg:tt)+) => {{
        let logger = crate::utils::logger::logger();
        slog::error!(logger, $msg, $($arg)+);
    }};
}