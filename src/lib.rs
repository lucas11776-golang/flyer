#![feature(async_fn_traits)]
#![feature(unboxed_closures)]
#![feature(downcast_unchecked)]
#![feature(type_alias_impl_trait)]
#![feature(fn_traits)]
#![feature(const_trait_impl)]
#![feature(const_convert)]

pub mod request;
pub mod response;
pub mod ws;
pub mod router;
pub mod utils;
pub mod session;
pub mod view;
pub mod assets;
pub mod cookie;
pub mod validation;
pub mod server;

use crate::{server::Server, utils::server::TlsPathConfig};

#[allow(static_mut_refs)]
pub fn server<'a>(host: &str, port: i32) -> Server {
    return Server::new(host, port, None);
}

#[allow(static_mut_refs)]
pub fn server_tls<'a>(host: &str, port: i32, key_path: &str, cert_path: &str) -> Server {
    return Server::new(host, port, Some(TlsPathConfig::new(key_path, cert_path)));
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
        slog::warn!(logger, $msg; "level" => "warn" $(, $($k => $v),* )? );
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