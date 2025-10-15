pub mod request;
pub mod response;
pub mod ws;
pub mod router;
pub mod utils;
pub mod session;
pub mod view;
pub mod server;

use std::sync::Arc;
use std::sync::atomic::{
    AtomicBool,
    Ordering
};

use futures::FutureExt;
use tokio::runtime::Runtime;
use tokio_rustls::TlsAcceptor;

use crate::request::Request;
use crate::response::Response;
use crate::router::group::GroupRouter;
use crate::router::Router;
use crate::server::{get_server_config, RequestCallback};
use crate::server::tcp::NewTcpServer;
use crate::server::udp::UdpServer;
use crate::server::{
    TlsPathConfig,
    tcp::{TcpServer}
};
use crate::session::SessionManager;
use crate::utils::Configuration;



pub(crate) struct HttpConfig {

}

#[derive(Default)]
pub struct HTTP<'a> {
    pub(crate) host: String,
    pub(crate) port: i32,
    pub(crate) tls: Option<TlsPathConfig>,


    // acceptor: Option<TlsAcceptor>,
    pub(crate) request_max_size: i64,
    pub(crate) router: GroupRouter<'a>,
    pub(crate) session_manger: Option<Box<dyn SessionManager>>,
    pub(crate) configuration: Configuration,
}

fn new_http<'a>(host: &str, port: i32, tls: Option<TlsPathConfig>) -> HTTP<'a> {

    return HTTP {
        host: host.to_owned(),
        port: port,
        tls: tls,
        request_max_size: 1024,
        router: GroupRouter::new(),
        session_manger: None,
        configuration: Configuration::new(),
    };
}

pub fn server<'a>(host: &'a str, port: i32) -> HTTP<'a> {
    return new_http(host, port, None);
}

pub fn server_tls<'a>(host: &'a str, port: i32, key: &str, cert: &str) -> HTTP<'a> {
    return new_http(host, port, Some(TlsPathConfig {
        key_path: key.to_owned(),
        cert_path: cert.to_owned()
    }));
}

impl <'a>HTTP<'a> {
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

    pub fn view(&'a mut self, path: &str) -> &mut HTTP {
        self.configuration.insert("view_path".to_owned(), path.to_owned());

        return self;
    }

    pub fn session(&'a mut self, manager: Box<dyn SessionManager>) -> &mut HTTP {
        self.session_manger = Some(manager);

        return self;
    }

    pub fn listen(&'a mut self) {
        // Runtime::new().unwrap().block_on(async {
        //     tokio_scoped::scope(|scope| {
        //         scope.spawn(self.tcp_server());
        //     });

        //     tokio_scoped::scope(|scope| {
        //         scope.spawn(self.udp_server());
        //     });
        // });


        Runtime::new().unwrap().block_on(async {
            tokio_scoped::scope(|scope| {
                scope.spawn(self.tcp_server());
            });

            // tokio_scoped::scope(|scope| {
            //     scope.spawn(self.udp_server());
            // });
        });

        // // TODO: check if the is no better way...
        // self.block_main_thread();
    }


    async fn handle_request(&mut self, req: Request, res: Response) {

    }

    async fn tcp_server(&'a mut self) {

            
        let mut acceptor: Option<TlsAcceptor> = None;

        if self.tls.is_some() {
            acceptor = Some(TlsAcceptor::from(Arc::new(get_server_config(self.tls.as_ref().unwrap()).unwrap())));
        }

        NewTcpServer::new(self.host.to_string(), self.port, acceptor.clone()).await.unwrap()
            .on_request(async |req, res| self.handle_request(req, res).await)
            .await;

        // TcpServer::new(self).await
        //     .listen()
        //     .await;
    }

    async fn udp_server(&'a mut self) {
        UdpServer::new(self).await
            .listen()
            .await;
    }

    pub fn router(&'a mut self) -> Router<'a> {
        return Router{
            router: &mut self.router,
            path: vec!["/".to_string()],
            middleware: vec![],
            get: None,
        };
    }

    fn block_main_thread(&mut self) {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone: Arc<AtomicBool> = running.clone();

        ctrlc::set_handler(move || {
            running_clone.store(false, Ordering::SeqCst);
        }).unwrap();

        while running.load(Ordering::SeqCst) {}
    }
}