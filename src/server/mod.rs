pub mod udp;
pub mod tcp;
pub mod handler;

use std::{io::Result as IOResult};

use rustls::{
    ServerConfig,
    pki_types::{
        pem::{PemObject},
        CertificateDer,
        PrivateKeyDer
    }
};

use crate::{
    request::Request, response::Response, view::new_view, ws::Ws, HTTP
};

type Protocol<'a> = &'a str;

const HTTP1: Protocol = "HTTP/1.1";
const HTTP2: Protocol = "HTTP/2.0";
const HTTP3: Protocol = "HTTP/3.0";

pub trait Server<'a> {
    fn new(http: &'a mut HTTP) -> &'a mut Self;
    fn listen() -> IOResult<()>;
}

pub struct TlsConfig { 
    pub key: PrivateKeyDer<'static>,
    pub cert: Vec<CertificateDer<'static>>
}

#[derive(Debug)]
pub struct TlsPathConfig {
    pub key_path: String,
    pub cert_path: String
}


pub fn get_tls_config(tls: &TlsPathConfig) -> IOResult<TlsConfig> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();

    return Ok(TlsConfig {
        key: PrivateKeyDer::from_pem_file(tls.key_path.clone())
            .unwrap(),
        cert: CertificateDer::pem_file_iter(tls.cert_path.clone())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    })
}

pub fn get_server_config(tls: &TlsPathConfig) -> IOResult<ServerConfig> {
    let config = get_tls_config(tls)?;
    return Ok(
        rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(config.cert, config.key)
        .unwrap()
    );
}