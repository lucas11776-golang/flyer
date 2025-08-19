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


mod handler;

use crate::zero::handler::http1x;
use crate::zero::router::{NewRouter, Router};

pub(crate) mod response;
pub(crate) mod request;
pub(crate) mod router;

pub struct HTTP {
    acceptor: Option<Arc<SslAcceptor>>,
    listener: TcpListener,
    request_max_size: i64,
    is_secure: bool,
    router: Router,
}

pub fn server(host: String, port: i32) -> Result<HTTP> {
    let http = HTTP {
        listener: TcpListener::bind(format!("{0}:{1}", host, port))?,
        request_max_size: 1024,
        acceptor: None,
        is_secure: false,
        router: NewRouter(),
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
        is_secure: true,
        router: NewRouter(),
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

    fn handle_request_http_2_x(&mut self, req: &mut request::Request) {
        
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
        if self.is_secure {
            self.listen_secure()
        } else {
            self.listen_none_secure()
        }
    }

    fn listen_secure(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    let _ = scope(|_s| {
                        match self.acceptor.clone().unwrap().accept(socket) {
                            Ok(socket) => self.new_connection(socket, addr),
                            Err(_error) => Ok(()),
                        }
                    }).unwrap();
                },
                Err(err) => println!("Error: {0}", err),
            }
        }
    }

    fn listen_none_secure(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    let _ = scope(|_s| {
                        self.new_connection(socket, addr)
                    });
                },
                Err(err) => println!("Error: {0}", err),
            }
        }
    }

    pub fn router(&mut self) -> &mut Router {
        return &mut self.router;
    }
}