pub mod request;
pub mod handler;
pub mod response;
pub mod ws;
pub mod router;
pub mod utils;
pub mod session;
pub mod view;
pub mod server;

use std::collections::HashMap;
use std::io::{Result as IOResult};
use std::net::SocketAddr;
use std::pin::{pin};
use std::sync::Arc;

use tokio::net::{TcpListener};
use tokio::runtime::Runtime;
use tokio_rustls::{TlsAcceptor};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::handler::{http1, http2};
use crate::handler::http2::{H2_PREFACE};
use crate::request::Request;
use crate::response::Response;
use crate::router::{new_group_router, GroupRouter, Router};
use crate::server::{get_server_config, RoutesCallback, WebCallback};
use crate::server::tcp::{TcpServer, Tls};
use crate::session::SessionManager;
use crate::ws::Ws;


use std::{thread, time::Duration};
use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;


pub type Values = HashMap<String, String>;
pub type Configuration = HashMap<String, String>;

pub struct HTTP {
    // acceptor: Option<TlsAcceptor>,
    // listener: TcpListener,
    host: String,
    port: i32,
    tls: Option<Tls>,
    request_max_size: i64,
    router: GroupRouter,
    pub(crate) session_manger: Option<Box<dyn SessionManager>>,
    pub(crate) configuration: Configuration,
}

pub async fn server<'a>(host: &str, port: i32) -> IOResult<HTTP> {
    return Ok(HTTP {
        // acceptor: None,
        // listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        host: host.to_owned(),
        port: port,
        tls: None,
        request_max_size: 1024,
        router: new_group_router(),
        session_manger: None,
        configuration: Configuration::new()
    });
}

pub async fn server_tls<'a>(host: &str, port: i32, key: &str, cert: &str) -> IOResult<HTTP> {
    return Ok(HTTP {
        // acceptor: Some(TlsAcceptor::from(Arc::new(get_server_config(key, certs)?))),
        // listener: TcpListener::bind(format!("{0}:{1}", host, port)).await?,
        host: host.to_owned(),
        port: port,
        tls: Some(Tls {
            key_path: key.to_owned(),
            cert_path: cert.to_owned()
        }),
        request_max_size: 1024,
        router: new_group_router(),
        session_manger: None,
        configuration: Configuration::new(),
    });
}

// TODO: find better way

// fn web_handler<'a>(http, req: &'a mut Request, res: &'a mut Response) {

// }

impl HTTP {
    pub fn host(&self) -> String {
        return "".to_owned();  //self.listener.local_addr().unwrap().ip().to_string();
    }

    pub fn port(&self) -> i32 {
        return 9999 ;//self.listener.local_addr().unwrap().port().into();
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

    pub fn session(&mut self, manager: impl SessionManager) -> &mut HTTP {
        return self;
    }

    async fn handle_stream<RW>(&mut self, stream: RW, addr:  SocketAddr) -> IOResult<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        let mut reader = pin!(BufReader::new(stream));
        let buf = reader.fill_buf().await?;

        // match buf.len() >= H2_PREFACE.len() && &buf[..H2_PREFACE.len()] == H2_PREFACE {
        //     true => http2::Handler::handle(self,reader, addr).await?,
        //     false => http1::Handler::handle(self,reader, addr).await?,
        // }

        Ok(())
    }

    async fn handle_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        match self.handle_stream(stream, addr).await {
            Ok(_) => {},
            Err(err) => println!("error: {}", err),
        }
    }

    async fn new_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        // match &self.acceptor {
        //     Some(acceptor) => {
        //         let _ = match acceptor.accept(stream).await {
        //             Ok(stream) => self.handle_connection(stream, addr).await,
        //             Err(err) => println!("error: {}", err),
        //         };
        //     },
        //     None => self.handle_connection(stream, addr).await,
        // };
    }

  


    fn web_callback<'a>(http: &HTTP,  req: &'a mut Request, res: &'a mut Response) {
        Runtime::new().unwrap().block_on(async {
            // http.address();
        });
    } 

    async fn ws_callback<'a>(server: &mut HTTP, req: &'a mut Request, res: &'a mut Ws) {
        Runtime::new().unwrap().block_on(async {
                
        });
    }

    pub async fn tcp_server(&mut self) {
        TcpServer::new(&self.host, self.port, self.tls.clone()).await
            .listen()
            .await;
    }

    pub async fn udp_server(&mut self) {
        
    }

    pub async fn listen(&mut self) -> IOResult<()> {
        // loop {
        //     match self.listener.accept().await {
        //         Ok((stream, addr)) => {
        //             tokio_scoped::scope(|scope| {
        //                 scope.spawn(self.new_connection(stream, addr));
        //             });
        //         },
        //         Err(err) => println!("{}", err),
        //     }
        // }

        match &self.tls {
            Some(tls) => {
                // tokio_scoped::scope(|scope| {
                //     scope.spawn(self.udp_server(None));
                // });
            },
            None => {
                tokio_scoped::scope(|scope| {
                    scope.spawn(self.tcp_server());
                });
            },
        }


        self.block_main_thread();

        Ok(())
    }

    pub fn router(&mut self) -> Router {
        return Router{
            router: &mut self.router,
            path: vec!["/".to_string()],
            middleware: vec![],
        };
    }

    fn block_main_thread(&mut self) {
        let running = Arc::new(AtomicBool::new(true));
        let c_running: Arc<AtomicBool> = running.clone();

        ctrlc::set_handler(move || {
            c_running.store(false, Ordering::SeqCst);
        }).unwrap();

        while running.load(Ordering::SeqCst) {}
    }
}