pub(crate) mod handler;
pub(crate) mod response;
pub(crate) mod request;
pub(crate) mod router;

use std::net::{SocketAddr, TcpListener};
use std::io::{Read, Result, Write};
use std::sync::Arc;
use std::thread::{scope};

use openssl::ssl::{
    SslAcceptor,
    SslAcceptorBuilder,
    SslFiletype,
    SslMethod
};

use crate::handler::http1x;
use crate::router::{new_router, Router};

pub struct HTTP {
    acceptor: Option<Arc<SslAcceptor>>,
    listener: TcpListener,
    request_max_size: i64,
    router: Router,
}

pub fn server(host: String, port: i32) -> Result<HTTP> {
    let http = HTTP {
        listener: TcpListener::bind(format!("{0}:{1}", host, port))?,
        request_max_size: 1024,
        acceptor: None,
        router: new_router(),
    };
    return Ok(http);
}

pub fn server_tls(host: String, port: i32, key: String, certs: String) -> Result<HTTP> {
    let mut acceptor: SslAcceptorBuilder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

    acceptor.set_private_key_file(key, SslFiletype::PEM)?;
    acceptor.set_certificate_chain_file(certs)?;
    acceptor.check_private_key()?;

    let http = HTTP {
        acceptor: Some(Arc::new(acceptor.build())),
        listener: TcpListener::bind(format!("{0}:{1}", host, port))?,
        request_max_size: 1024,
        router: new_router(),
    };

    return Ok(http);
}

impl HTTP {
    pub fn host(&self) -> String {
        return self.listener.local_addr().unwrap().ip().to_string();
    }

    pub fn port(&self) -> i32 {
        return self.listener.local_addr().unwrap().port().into();
    }

    pub fn address(&self) -> String {
        return std::format!("{0}:{1}", self.host(), self.port());
    }

    pub fn set_request_max_size(&mut self, size: i64) {
        self.request_max_size = size;    
    }

    fn new_connection<'a, T: Write + Read>(&mut self, mut socket: T, mut _addr: SocketAddr) -> Result<()> {
        let mut buffer: [u8; 1024] = [0; 1024];
        let size = socket.read( &mut buffer)?;
        // TODO: Net to check here if request if HTTP/1.x or HTTP/2.0
        let mut req = request::parse(String::from_utf8_lossy(&buffer[0..size]).to_string())?;
    
        let _ = match req.protocol.to_uppercase() {
            protocol if protocol == "HTTP/1.1" => http1x::handle(self, socket, &mut req),
            // protocol if protocol == "HTTP/2.1" => self.handle_request_http_2_x(&mut req)?,
            _default => panic!("Protocol {} not support", req.protocol)
        };

        return Ok(());
    }

    pub fn listen(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    match &self.acceptor {
                        Some(acceptor) => match acceptor.accept(socket) {
                            Ok(socket) => scope(|_| self.new_connection(socket, addr)).unwrap(),
                            Err(err) => println!("{}", err),
                        },
                        None => match scope(|_| self.new_connection(socket, addr)) {
                            Ok(_) => (),
                            Err(err) => println!("{}", err),
                        },
                    };
                },
                Err(err) => println!("{}", err),
            }
        }
    }

    pub fn router(&mut self) -> &mut Router {
        return &mut self.router;
    }
}