pub mod udp;
pub mod tcp;
pub mod handler;
pub mod helpers;

use std::{io::Result as IoResult, sync::Arc};

use rustls::{
    ServerConfig,
    pki_types::{
        pem::PemObject,
        CertificateDer,
        PrivateKeyDer
    }
};
use tokio_rustls::TlsAcceptor;

use crate::HTTP;

pub enum Protocol {
    HTTP1,
    HTTP2,
    HTTP3
}

pub trait Server<'a> {
    fn new(http: &'a mut HTTP) -> &'a mut Self;
    fn listen() -> IoResult<()>;
}

pub struct TlsConfig { 
    pub key: PrivateKeyDer<'static>,
    pub cert: Vec<CertificateDer<'static>>
}

pub struct TlsPathConfig {
    pub key_path: String,
    pub cert_path: String
}

pub fn get_tls_config(tls: &TlsPathConfig) -> IoResult<TlsConfig> {
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

pub fn get_server_config(tls: &TlsPathConfig) -> IoResult<ServerConfig> {
    return server_config(get_tls_config(tls)?);
}

pub fn server_config(config: TlsConfig) -> IoResult<ServerConfig> {
    return Ok(
        rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(config.cert, config.key)
        .unwrap()
    );
}

pub fn get_tls_acceptor(config: ServerConfig) -> Option<TlsAcceptor> {
    return Some(TlsAcceptor::from(Arc::new(config)));
}