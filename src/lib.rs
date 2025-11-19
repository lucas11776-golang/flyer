#![feature(async_fn_traits)]
#![feature(unboxed_closures)]
#![feature(downcast_unchecked)]

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

use crate::{http::HTTP, server::TlsPathConfig};

pub fn server(host: &str, port: i32) -> HTTP {
    return HTTP::new(host, port, None);
}

pub fn server_tls(host: &str, port: i32, key: &str, cert: &str) -> HTTP {
    return HTTP::new(host, port, Some(TlsPathConfig {
        key_path: key.to_owned(),
        cert_path: cert.to_owned()
    }));
}