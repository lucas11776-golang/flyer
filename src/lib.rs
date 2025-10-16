pub mod request;
pub mod response;
pub mod ws;
pub mod router;
pub mod utils;
pub mod session;
pub mod view;
pub mod server;

use std::io::Result;
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
use crate::router::{Route, Router, WebRoute, WsRoute};
use crate::server::{get_server_config, HttpRequestCallback};
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
pub struct HTTP {
    pub(crate) host: String,
    pub(crate) port: i32,
    pub(crate) tls: Option<TlsPathConfig>,
    // acceptor: Option<TlsAcceptor>,
    pub(crate) request_max_size: i64,
    // pub(crate) router: GroupRouter<'a>,
    pub(crate) session_manger: Option<Box<dyn SessionManager>>,
    pub(crate) configuration: Configuration,
}

fn new_http(host: &str, port: i32, tls: Option<TlsPathConfig>) -> HTTP {
    return HTTP {
        host: host.to_owned(),
        port: port,
        tls: tls,
        // acceptor: acceptor,
        request_max_size: 1024,
        // router: GroupRouter::new(),
        session_manger: None,
        configuration: Configuration::new(),
    };
}

pub fn server<'a>(host: &str, port: i32) -> HTTP {
    return new_http(host, port, None);
}

pub fn server_tls<'a>(host: &'a str, port: i32, key: &str, cert: &str) -> HTTP {
    return new_http(host, port, Some(TlsPathConfig {
        key_path: key.to_owned(),
        cert_path: cert.to_owned()
    }));
}



impl <'a>HTTP {
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

    pub fn view(&mut self, path: &str) -> &mut HTTP {
        self.configuration.insert("view_path".to_owned(), path.to_owned());

        return self;
    }

    pub fn session(&mut self, manager: Box<dyn SessionManager>) -> &mut HTTP {
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
        HTTP::block_main_thread();
    }


    pub async fn ws_request(&mut self, req: &mut Request, res: &mut Response) -> Option<&mut Route<WsRoute>>
    where
    {

        return None
    }


    fn get_tls_acceptor(&mut self) -> Result<Option<TlsAcceptor>> {
        Ok(
            match self.tls.as_mut() {
                Some(tls) => Some(TlsAcceptor::from(Arc::new(get_server_config(tls).unwrap()))),
                None => None,
            }
        )
    }

    async fn tcp_server<'s>(&'a mut self)
    where 
        'a: 's
     {
        let mut server = NewTcpServer::new(self.host.to_string(), self.port, self.get_tls_acceptor().unwrap()).await.unwrap();

        // {
        //     server.http_request(async |req, res| self.router.match_web_routes(req, res).await).await;
        // }

        server.listen().await;
    }

    async fn udp_server(&'a mut self) {
        // UdpServer::new(self).await
        //     .listen()
        //     .await;
    }

    pub fn router(&'a mut self) -> Router<'a>
    {
        return Router {
            // router: &mut self.router,
            path: vec!["/".to_string()],
            middleware: vec![],
            get: None,
        };
    }

    fn block_main_thread() {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone: Arc<AtomicBool> = running.clone();

        ctrlc::set_handler(move || {
            running_clone.store(false, Ordering::SeqCst);
        }).unwrap();

        while running.load(Ordering::SeqCst) {}
    }
}