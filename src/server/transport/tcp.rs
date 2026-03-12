use std::pin::Pin;
use std::net::SocketAddr;

use anyhow::Result;
use h2::server;
use tokio::{io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader}, net::TcpListener};
use tokio_rustls::TlsAcceptor;

use crate::server::transport::handler::http2;
use crate::{GLOBAL_SERVER, response::Response, server::transport::handler::http2::H2_PREFACE};
use crate::{server::{Server, transport::{Protocol, handler::http1}}, utils::server::get_tls_acceptor, warn};

pub async fn listen(server_ptr: usize) {
    unsafe {
        listener(
            TcpListener::bind((*(server_ptr as *const &mut Server)).address()).await.unwrap(),
            &(*(server_ptr as *const &mut Server)).server_config.clone().map(|config| get_tls_acceptor(config.clone()).unwrap()) as *const Option<TlsAcceptor> as usize
        ).await;
    }
}

async fn listener(listener: TcpListener, tls_ptr: usize) {
    unsafe  {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(async move {
                match &*(tls_ptr as *const Option<TlsAcceptor>) {
                    Some(acceptor) => {
                        match acceptor.accept(stream).await {
                            Ok(rw) => connection(BufReader::new(rw), addr).await,
                            Err(err) => warn!("TLS connection error"; "error" => err),
                        }
                    },
                    None => {
                        connection(BufReader::new(stream), addr).await
                    },
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

async fn connection<RW>(mut rw: BufReader<RW>, addr: SocketAddr)
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    match connection_protocol(&mut rw).await {
        Ok(protocol) => {
            let result = match protocol {
                Protocol::HTTP1 => { http_1_protocol(rw, addr).await },
                Protocol::HTTP2 => { http_2_protocol(rw, addr).await },
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

#[allow(static_mut_refs)]
async fn http_1_protocol<RW>(mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
{
    unsafe {
        let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
        let handle_result = handler.handle().await;

        if let Err(err) = handle_result {
            return Err(err);
        }

        let mut req = handle_result.unwrap();
        let mut res = Response::new();

        if req.header("upgrade").to_lowercase() == "websocket" {
            // let (mut req, mut res) = RequestHandler::new().setup(req, res).await.unwrap();

            // handle_web_socket(rw, &mut req, &mut res).await.unwrap();

            println!("Socket Request...");

            todo!()
        }

        GLOBAL_SERVER.get_mut().unwrap().on_request(&mut req, &mut res).await.unwrap(); // TODO: need to remove and use `server_ptr`
        handler.write(&mut req, &mut res).await.unwrap();

        drop(rw);

        return Ok(());
    }
}

#[allow(static_mut_refs)]
async fn http_2_protocol<RW>(rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    unsafe {
        match server::handshake(rw).await {
            Ok(mut conn) => {
                 while let Some(result) = conn.accept().await {
                    match result {
                        Ok((request, send)) => {
                            tokio::spawn(async move {
                                let mut handler = http2::Handler::new(addr, send);
                                let (mut req, mut res) = (handler.transform(request).await.unwrap(), Response::new());

                                GLOBAL_SERVER.get_mut().unwrap().on_request(&mut req, &mut res).await.unwrap(); // TODO: need to remove and use `server_ptr`

                                handler.write(&mut req, &mut res).await.unwrap();
                            });
                        },
                        Err(_) => {},
                    }
                }
                return Ok(());
            },
            Err(err) => {
                return Err(err.into());
            },
        }
    }
}
