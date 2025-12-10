use std::sync::LazyLock;
use std::{io::Result, net::SocketAddr};
use std::pin::Pin;

use h2::server;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::request::Request;
use crate::response::Response;
use crate::server::handler::{http1, http2, ws_http1};
use crate::server::handler::http2::H2_PREFACE;
use crate::server::helpers::{setup, teardown};
use crate::server::protocol::HttpProtocol;
use crate::server::protocol::http::GLOBAL_HTTP;
use crate::utils::server::get_tls_acceptor;
use crate::warn;

static mut TLS_ACCEPTOR: LazyLock<Option<TlsAcceptor>> = LazyLock::new(|| None);

#[allow(static_mut_refs)]
pub(crate) async fn listen() {
    unsafe {
        if let Some(config) = &GLOBAL_HTTP.server_config {
            #[allow(unused)]
            TLS_ACCEPTOR.insert(get_tls_acceptor(config.clone()).unwrap());
        }

        listen_connection(TcpListener::bind(format!("{}", GLOBAL_HTTP.address())).await.unwrap()).await;
    }
}

#[allow(static_mut_refs)]
async fn listen_connection(listener: TcpListener) {
    while let Ok((stream, addr)) = listener.accept().await  {
        tokio::spawn(async move {
            match unsafe { TLS_ACCEPTOR.as_mut() } {
                Some(acceptor) => {
                    match acceptor.accept(stream).await {
                        // TODO: Handle unwrap
                        Ok(rw) => connection(BufReader::new(rw), addr).await.unwrap(),
                        Err(err) => { warn!("TLS connection error"; "error" => err); },
                    }
                },
                // TODO: Handle unwrap
                None => connection(BufReader::new(stream), addr).await.unwrap(),
            }
        });
    }
}

fn get_connection_protocol(buffer: &[u8]) -> HttpProtocol {
    match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
        true => HttpProtocol::HTTP2,
        false => HttpProtocol::HTTP1,
    }
}

async fn connection<RW>(mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    Ok(
        match get_connection_protocol(rw.fill_buf().await?) {
            HttpProtocol::HTTP2 => http_2_protocol(rw, addr).await.unwrap(),
            _ => http_1_protocol(rw, addr).await.unwrap()
        }
    )
}

#[allow(static_mut_refs)]
async fn handle<'h>(mut req: Request, mut res: Response) -> Result<(Request, Response)> {
    unsafe {
        (req, res) = setup(req, res).await.unwrap();

        res.request_headers = req.headers.clone();

        let resp = GLOBAL_HTTP.router.web_match(&mut req, &mut res).await;

        if resp.is_none() && GLOBAL_HTTP.assets.is_some() {
            (req, res) = GLOBAL_HTTP.assets.as_mut().unwrap().handle(req, res).unwrap();
        }

        return Ok(teardown(req, res).await.unwrap());
    }
}

#[allow(static_mut_refs)]
async fn handle_web_socket<RW>(rw: BufReader<RW>, req: &mut Request, res: &mut Response) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    unsafe {
        let (mut handler, req, res) = ws_http1::Handler::new(rw, req, res).await.unwrap();
        let result = GLOBAL_HTTP.router.ws_match(req, res).await;

        if result.is_none() {
            return Ok(())
        }

        let (route, req, res) = result.unwrap();

        return Ok(handler.handle(route, req, res).await.unwrap());
    }
}

async fn http_1_protocol<RW>(mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
    let result = handler.handle().await;

    if result.is_none() {
        return Ok(());
    }

    let mut req = result.unwrap().unwrap();
    let mut res = Response::new();

    if req.header("upgrade") == "websocket" {
        (req, res) = setup(req, res).await.unwrap();

        handle_web_socket(rw, &mut req, &mut res).await.unwrap();

        return Ok(())
    }

    (req, res) = handle(req, res).await.unwrap();

    return Ok(handler.write(&mut req, &mut res).await.unwrap());
}

async fn http_2_protocol<RW>(rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    let mut conn = server::handshake(rw).await.unwrap();

    while let Some(result) = conn.accept().await {
        if result.is_err() {
            continue;
        }

        tokio::spawn(async move {
            let (request, send) = result.unwrap();
            let mut handler = http2::Handler::new(addr, send);
            let mut req = handler.handle(request).await.unwrap();
            let mut res = Response::new();

            (req, res) = handle(req, res).await.unwrap();

            handler.write( &mut req, &mut res).await.unwrap();
        });
    }

    return Ok(())   
}