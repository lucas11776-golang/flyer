
use std::net::SocketAddr;

use anyhow::Result;
use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{server::Server, utils::mem::Instance};

pub mod tcp;
pub mod udp;

#[allow(unused)]
pub enum Protocol {
    HTTP1,
    HTTP2,
    HTTP3
}

pub trait ServerHandler {
    async fn listen(instance: Instance<Server>);
}

pub trait TcpHandler {
    fn new(server: Instance<Server>, addr: SocketAddr) -> Self;

    async fn handle<RW>(&mut self, rw: BufReader<RW>) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static;
}

pub trait UdpHandler {
    fn new(server: Instance<Server>, addr: SocketAddr) -> Self;

    async fn handle(&mut self, connection: h3::server::Connection<h3_quinn::Connection, Bytes>) -> Result<()>;
}