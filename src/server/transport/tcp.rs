use std::{pin::Pin};
use std::net::SocketAddr;

use anyhow::Result;
use h2::server;
use tokio::{io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader}, net::TcpListener};
use tokio_rustls::TlsAcceptor;

use crate::server::transport::handler::ws;
use crate::{request::Request, server::transport::handler::http2};
use crate::{response::Response, server::transport::handler::http2::H2_PREFACE};
use crate::{server::{Server, transport::{Protocol, handler::http1}}, utils::server::get_tls_acceptor, warn};

pub async fn listen(ptr: usize) {
    let socket = TcpListener::bind(Server::instance(ptr).address()).await.unwrap();
    let tls= Server::instance(ptr).server_config.clone().map(|config| get_tls_acceptor(config.clone()).unwrap());

    listener(ptr, socket, tls).await;
}

async fn listener(ptr: usize, listener: TcpListener, tls: Option<TlsAcceptor>) {
    unsafe  {
        let tls_ptr = &tls as *const Option<TlsAcceptor> as usize;

        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(async move {
                match &*(tls_ptr as *const Option<TlsAcceptor>) {
                    Some(acceptor) => {
                        match acceptor.accept(stream).await {
                            Ok(rw) => connection(ptr, BufReader::new(rw), addr).await,
                            Err(err) => warn!("TLS connection error"; "error" => err),
                        }
                    },
                    None => { connection(ptr, BufReader::new(stream), addr).await },
                }
            });
        }
    }
}

async fn connection_protocol<RW>(rw: &mut BufReader<RW>) -> Result<Protocol>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    let buffer = rw.fill_buf().await.unwrap();

    Ok(
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => Protocol::HTTP2,
            false => Protocol::HTTP1,
        },
    )
}

async fn connection<RW>(ptr: usize, mut rw: BufReader<RW>, addr: SocketAddr)
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    match connection_protocol(&mut rw).await {
        Ok(protocol) => {
            let result = match protocol {
                Protocol::HTTP1 => { http_1_protocol(ptr, rw, addr).await },
                Protocol::HTTP2 => { http_2_protocol(ptr, rw, addr).await },
                Protocol::HTTP3 => { Err(anyhow::anyhow!("Unsupported request tcp connection")) },
            };

            if let Err(err) = result {
                warn!("failed to process request"; "error" => err);
            }
        }
        Err(err) => {
            warn!("request protocol error"; "error" => err);
        }
    };
}

async fn ws_protocol<RW>(ptr: usize, rw: BufReader<RW>, req: &mut Request, res: &mut Response) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    return Ok(
        match Server::instance(ptr).on_ws_request(req, res).await {
            Some((route, req, res)) => {
                ws::Handler::new(rw, req, res).await.unwrap().handle(route).await.unwrap();
            },
            None => { drop(rw); },
        }
    );
}

async fn http_1_protocol<RW>(ptr: usize, mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
{
    unsafe {
        let mut handler = http1::Handler::new(ptr, Pin::new(&mut rw), addr);
        let handle_result = handler.handle().await;

        if let Err(err) = handle_result {
            return Err(err);
        }

        let (mut req, mut res) = (handle_result.unwrap(), Response::new());

        if req.header("upgrade").to_lowercase() == "websocket" {
            return ws_protocol(ptr, rw, &mut req, &mut res).await;
        }

        Server::instance(ptr).on_web_request(&mut req, &mut res).await.unwrap();
        handler.write(&mut req, &mut res).await.unwrap();

        drop(rw);

        return Ok(());
    }
}

async fn http_2_protocol<RW>(ptr: usize, rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    match server::handshake(rw).await {
        Ok(mut conn) => {
                while let Some(result) = conn.accept().await {
                match result {
                    Ok((request, send)) => {
                        tokio::spawn(async move {
                            let mut handler = http2::Handler::new(addr, send);
                            let (mut req, mut res) = (handler.handle(request).await.unwrap(), Response::new());

                            Server::instance(ptr).on_web_request(&mut req, &mut res).await.unwrap();

                            handler.write(&mut req, &mut res).await.unwrap();
                        });
                    },
                    Err(_) => {},
                }
            }
            return Ok(());
        },
        Err(err) => { return Err(err.into()); },
    }
}