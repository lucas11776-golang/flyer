
use std::io::Result;
use std::mem::transmute_copy;

use futures::join;
use rustls::ServerConfig;
use tokio::runtime::Runtime;

use crate::assets::Assets;
use crate::router::group::GroupRouter;
use crate::router::Router;
use crate::server::{get_tls_config, server_config};
use crate::server::tcp::TcpServer;
use crate::server::udp::UdpServer;
use crate::server::TlsPathConfig;
use crate::session::SessionManager;
use crate::view::View;

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

impl HTTP {
    pub fn new(host: &str, port: i32, tls: Option<TlsPathConfig>) -> HTTP {
        return Self {
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

    pub fn router<'a>(&mut self) -> &mut Router {
        let idx = self.router.nodes.len();

        self.router.nodes.push(Box::new(Router::new()));

        return &mut self.router.nodes[idx];
    }

    pub fn listen(&mut self) {
        self.router.setup();

        let (udp_server_config, tcp_server_config) = self.get_servers_config().unwrap();
        let (udp_server_http, tcp_server_http) = self.get_server_http().unwrap();

        Runtime::new().unwrap().block_on(async {
            join!(
                HTTP::udp_server(udp_server_http, udp_server_config),
                HTTP::tcp_server(tcp_server_http, tcp_server_config),
            );
        });
    }

    fn get_servers_config(&mut self) -> Result<(Option<ServerConfig>, Option<ServerConfig>)> {
        let mut server_config_one: Option<ServerConfig> = None;
        let mut server_config_two: Option<ServerConfig> = None;

        if self.tls.is_some() {
            server_config_one = Some(server_config(get_tls_config(&self.tls.as_mut().unwrap())?)?);
            server_config_two = unsafe { transmute_copy(&server_config_one) };
        }

        return Ok((server_config_one, server_config_two));
    }

    fn get_server_http(&mut self) -> Result<(&mut HTTP, &mut HTTP)> {
        return Ok((unsafe{ transmute_copy(&self) }, self));
    }

    async fn tcp_server(http: &mut HTTP, config: Option<ServerConfig>) {
        TcpServer::new(http, config)
            .await
            .unwrap()
            .listen()
            .await;
    }

    async fn udp_server(http: &mut HTTP, config: Option<ServerConfig>) {
        if config.is_none() {
            return;
        }

        UdpServer::new(http, config.unwrap())
            .await
            .unwrap()
            .listen()
            .await;
    }

}