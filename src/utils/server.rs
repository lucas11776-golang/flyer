use std::{
    io::Result as IoResult,
    sync::Arc
};

use rustls::{
    ServerConfig,
    pki_types::{
        pem::PemObject,
        CertificateDer,
        PrivateKeyDer
    }
};
use tokio_rustls::TlsAcceptor;

pub(crate) struct TlsConfig { 
    pub key: PrivateKeyDer<'static>,
    pub cert: Vec<CertificateDer<'static>>
}

pub(crate) struct TlsPathConfig {
    pub key_path: String,
    pub cert_path: String
}

impl TlsPathConfig {
    pub fn new(key_path: &str, cert_path: &str) -> TlsPathConfig {
        return Self {
            key_path: String::from(key_path),
            cert_path: String::from(cert_path)
        };
    }
}

impl TlsConfig {
    pub fn new(key: PrivateKeyDer<'static>, cert: Vec<CertificateDer<'static>>) -> Self {
        return Self {
            key: key,
            cert: cert
        };
    }
}

pub(crate) fn get_tls_config(tls: &TlsPathConfig) -> IoResult<TlsConfig> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap();

    let key = PrivateKeyDer::from_pem_file(tls.key_path.clone()).unwrap();
    let cert = CertificateDer::pem_file_iter(tls.cert_path.clone())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    return Ok(TlsConfig::new(key, cert))
}

pub(crate) fn server_config(config: TlsConfig) -> IoResult<ServerConfig> {
    return Ok(
        rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(config.cert, config.key)
        .unwrap()
    );
}

pub(crate) fn get_tls_acceptor(config: ServerConfig) -> Option<TlsAcceptor> {
    return Some(TlsAcceptor::from(Arc::new(config)));
}