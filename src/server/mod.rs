pub mod udp;
pub mod tcp;

use std::io::{Result as IOResult};

use openssl::conf;
use rustls::{
    ServerConfig,
    pki_types::{
        pem::{PemObject},
        CertificateDer,
        PrivateKeyDer
    }
};

use crate::{
    request::Request,
    response::Response,
    ws::Ws
};

pub type WebCallback<'a> = fn(req: &'a mut Request, res: &'a mut Response); 
pub type WsCallback<'a> = fn(req: &'a mut Request, res: &'a mut Ws); 

pub struct RoutesCallback<'a> {
    pub web:  WebCallback<'a>,
    // pub ws: WsCallback<'a>
}

pub trait Server<'a> {
    fn new(host: &str, port: u32) -> Self;
    fn on_request(callbacks: RoutesCallback);
    fn listen() -> IOResult<()>;
}

pub struct TlsConfig { 
    pub key: PrivateKeyDer<'static>,
    pub cert: Vec<CertificateDer<'static>>
}

pub fn get_tls_config(key: &str, certs: &str) -> IOResult<TlsConfig> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();

    return Ok(TlsConfig {
        key: PrivateKeyDer::from_pem_file(key)
            .unwrap(),
        cert: CertificateDer::pem_file_iter(certs)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    })
}

pub fn get_server_config(key: &str, certs: &str) -> IOResult<ServerConfig> {
    let config = get_tls_config(key, certs)?;
    return Ok(
        rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(config.cert, config.key)
        .unwrap()
    );
}