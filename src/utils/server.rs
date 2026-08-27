use std::sync::Arc;
use std::path::Path;

use anyhow::Result;
use rustls::{
    ServerConfig,
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
};
use tokio_rustls::TlsAcceptor;

pub(crate) struct TlsConfig { 
    pub key: PrivateKeyDer<'static>,
    pub cert: Vec<CertificateDer<'static>>,
}

impl TlsConfig {
    #[inline]
    pub fn new(key: PrivateKeyDer<'static>, cert: Vec<CertificateDer<'static>>) -> Self {
        Self {
            key: key,
            cert: cert,
        }
    }
}

pub(crate) struct TlsPathConfig<'a> {
    pub key_path: &'a Path,
    pub cert_path: &'a Path,
}

impl<'a> TlsPathConfig<'a> {
    #[inline]
    pub fn new(key_path: &'a (impl AsRef<Path> + ?Sized), cert_path: &'a (impl AsRef<Path> + ?Sized)) -> Self {
        Self {
            key_path: key_path.as_ref(),
            cert_path: cert_path.as_ref(),
        }
    }
}

pub(crate) fn get_tls_config(tls: &TlsPathConfig) -> Result<TlsConfig> {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let key = PrivateKeyDer::from_pem_file(tls.key_path)?;
    let cert = CertificateDer::pem_file_iter(tls.cert_path)?
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(TlsConfig::new(key, cert))
}

pub(crate) fn server_config(config: TlsConfig) -> Result<ServerConfig> {
    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(config.cert, config.key)
        .map_err(Into::into)
}

#[inline]
pub(crate) fn get_tls_acceptor(config: ServerConfig) -> TlsAcceptor {
    TlsAcceptor::from(Arc::new(config))
}