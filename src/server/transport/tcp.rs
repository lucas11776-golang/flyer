use std::io::{ErrorKind, Result, Error};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::LazyLock;

use h2::server;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::request::Request;
use crate::response::Response;
use crate::server::handler::http2::H2_PREFACE;
use crate::server::handler::{http1, http1_ws, http2};
use crate::server::helpers::{Handler, RequestHandler};
use crate::server::protocol::Protocol;
use crate::server::protocol::http::APPLICATION;
use crate::utils::server::get_tls_acceptor;
use crate::warn;

static mut TLS_ACCEPTOR: LazyLock<Option<TlsAcceptor>> = LazyLock::new(|| None);

#[allow(static_mut_refs)]
pub(crate) async fn listen() {
    unsafe {
        if let Some(config) = &APPLICATION.server_config {
            #[allow(unused)]
            TLS_ACCEPTOR.insert(get_tls_acceptor(config.clone()).unwrap());
        }

        listener(
            TcpListener::bind(format!("{}", APPLICATION.address()))
                .await
                .unwrap(),
        )
        .await;
    }
}

#[allow(static_mut_refs)]
async fn listener(listener: TcpListener) {
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(async move {
            match unsafe { TLS_ACCEPTOR.as_mut() } {
                Some(acceptor) => match acceptor.accept(stream).await {
                    Ok(rw) => connection(BufReader::new(rw), addr).await,
                    Err(err) => warn!("TLS connection error"; "error" => err),
                },
                None => connection(BufReader::new(stream), addr).await,
            }
        });
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
                Protocol::HTTP3 => { Err(Error::new(ErrorKind::Unsupported, "Unsupported request tcp connection")) },
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
async fn handle_web_socket<RW>(
    rw: BufReader<RW>,
    req: &mut Request,
    res: &mut Response,
) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    unsafe {
        let (mut handler, req, res) = http1_ws::Handler::new(rw, req, res).await.unwrap();
        let result = APPLICATION.router.ws_match(req, res).await;

        if result.is_none() {
            return Ok(());
        }

        let (route, req, res) = result.unwrap();

        return Ok(handler.handle(route, req, res).await.unwrap());
    }
}

#[allow(static_mut_refs)]
async fn http_1_protocol<RW>(mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Sync + Send + 'static,
{
    unsafe {
        let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
        let handle_result = handler.handle().await;

        if handle_result.is_err() {
            return Err(handle_result.err().unwrap());
        }

        let req = handle_result.unwrap();
        let res = Response::new();

        if req.header("upgrade").to_lowercase() == "websocket" {
            let (mut req, mut res) = RequestHandler::new().setup(req, res).await.unwrap();

            handle_web_socket(rw, &mut req, &mut res).await.unwrap();

            return Ok(());
        }

        let (mut req, mut res) = APPLICATION.on_request(req, res).await.unwrap();

        return Ok(handler.write(&mut req, &mut res).await.unwrap());
    }
}

#[allow(static_mut_refs)]
async fn http_2_protocol<RW>(rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    unsafe {
        let mut conn = server::handshake(rw).await.unwrap();

        while let Some(result) = conn.accept().await {
            if result.is_err() {
                continue;
            }

            tokio::spawn(async move {
                let (request, send) = result.unwrap();
                let mut handler = http2::Handler::new(addr, send);
                let req = handler.handle(request).await.unwrap();
                let (mut req, mut res) = APPLICATION.on_request(req, Response::new()).await.unwrap();

                handler.write(&mut req, &mut res).await.unwrap();
            });
        }

        return Ok(());
    }
}
