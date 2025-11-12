#![feature(async_fn_traits)]
#![feature(unboxed_closures)]

pub mod request;
pub mod response;
pub mod ws;
pub mod router;
pub mod utils;
pub mod session;
pub mod view;
pub mod server;
pub mod assets;

use std::mem;

use futures::join;
use rustls::ServerConfig;
use tokio::runtime::Runtime;
use tokio_rustls::TlsAcceptor;

use crate::assets::Assets;
use crate::router::group::GroupRouter;
use crate::router::Router;
use crate::server::{get_tls_acceptor, get_tls_config, server_config};
use crate::server::tcp::TcpServer;
use crate::server::udp::UdpServer;
use crate::server::TlsPathConfig;
use crate::session::SessionManager;
use crate::view::View;

#[derive(Default)]
pub struct HTTP {
    pub(crate) host: String,
    pub(crate) port: i32,
    pub(crate) tls: Option<TlsPathConfig>,
    pub(crate) request_max_size: i64,
    pub(crate) router: GroupRouter,
    pub(crate) session_manager: Option<Box<dyn SessionManager>>,
    pub(crate) view: Option<View>,
    pub(crate) assets: Option<Assets>
}

fn new_http_server(host: &str, port: i32, tls: Option<TlsPathConfig>) -> HTTP {
    return HTTP {
        host: host.to_owned(),
        port: port,
        tls: tls,
        request_max_size: 1024,
        router: GroupRouter::new(),
        view: None,
        session_manager: None,
        assets: None,
    };
}

pub fn server(host: &str, port: i32) -> HTTP {
    return new_http_server(host, port, None);
}

pub fn server_tls(host: &str, port: i32, key: &str, cert: &str) -> HTTP {
    return new_http_server(host, port, Some(TlsPathConfig {
        key_path: key.to_owned(),
        cert_path: cert.to_owned()
    }));
}

impl HTTP {
    pub fn host(&self) -> String {
        return self.host.to_owned();
    }

    pub fn port(&self) -> i32 {
        return self.port;
    }

    pub fn address(&self) -> String {
        return std::format!("{0}:{1}", self.host(), self.port());
    }

    pub fn set_request_max_size(&mut self, size: i64) {
        self.request_max_size = size;    
    }

    pub fn view(mut self, path: &str) -> Self {
        self.view = Some(View::new(path));

        return self;
    }

    pub fn assets(mut self, path: &str, max_size_kilobytes: usize, expires_in_seconds: u128) -> Self {
        self.assets = Some(Assets::new(path.to_owned(), max_size_kilobytes, expires_in_seconds));

        return self;
    }

    pub fn session(mut self, manager: impl SessionManager + 'static) -> Self {
        self.session_manager = Some(Box::new(manager));

        return self;
    }

    pub fn router<'a>(&'a mut self) -> Router<'a> {
        return Router {
            router: &mut self.router,
            path: vec!["/".to_string()],
            middleware: vec![],
        };
    }

    pub fn listen(&mut self) {
        let udp_http: &mut HTTP = unsafe { mem::transmute_copy(&self) };
        let mut tls_server_config: Option<ServerConfig> = None;

        if self.tls.is_some() {
            tls_server_config = Some(server_config(get_tls_config(&self.tls.as_mut().unwrap()).unwrap()).unwrap());
        }

        let udp_tls_server_config: Option<ServerConfig> = unsafe {  mem::transmute_copy(&tls_server_config) };

        Runtime::new().unwrap().block_on(async {
            let udp_server = async {
                if udp_tls_server_config.is_none() {
                    return;
                }

                UdpServer::new(udp_http,  udp_tls_server_config.unwrap())
                            .await
                            .unwrap()
                            .listen()
                            .await;

            };

            let tcp_server = async {
                let mut acceptor: Option<TlsAcceptor> = None;

                if tls_server_config.is_some() {
                    acceptor = get_tls_acceptor(tls_server_config.unwrap());
                }
                
                TcpServer::new(self, acceptor)
                            .await
                            .unwrap()
                            .listen()
                            .await;
            };
            
            join!(tcp_server, udp_server);
        });
    }
}