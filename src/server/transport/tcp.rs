use std::io::Result;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::LazyLock;

use h2::server;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::request::Request;
use crate::response::Response;
use crate::server::handler::http2::H2_PREFACE;
use crate::server::handler::{http1, http1_ws, http2};
use crate::server::helpers::{setup, teardown};
use crate::server::protocol::Protocol;
use crate::server::protocol::http::GLOBAL_HTTP;
use crate::utils::async_peek::{AsyncPeek, Peek};
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

        listener(
            TcpListener::bind(format!("{}", GLOBAL_HTTP.address()))
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
                    Ok(rw) => connection(BufReader::new(Peek::new(rw)), addr).await,
                    Err(err) => warn!("TLS connection error"; "error" => err),
                },
                None => connection(BufReader::new(Peek::new(stream)), addr).await,
            }
        });
    }
}

async fn connection_protocol<RW>(rw: &mut BufReader<RW>) -> Result<Protocol>
where
    RW: AsyncPeek + Sync + Send + 'static,
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
    RW: AsyncPeek + Sync + Send + 'static,
{
    let connection_protocol = connection_protocol(&mut rw).await;

    if connection_protocol.is_err() {
        return warn!("request protocol error"; "error" => connection_protocol.err().unwrap());
    }

    let protocol = match connection_protocol.unwrap() {
        Protocol::HTTP2 => http_2_protocol(rw, addr).await,
        _ => http_1_protocol(rw, addr).await, // TODO: fix empty read error in header read...
    };

    if protocol.is_err() {
        return warn!("request handle error"; "error" => protocol.err().unwrap());
    }

    protocol.unwrap();
}

#[allow(static_mut_refs)]
async fn handle<'h>(mut req: Request, mut res: Response) -> Result<(Request, Response)> {
    unsafe {
        (req, res) = setup(req, res).await.unwrap();

        res.request_headers = req.headers.clone();

        let resp = GLOBAL_HTTP.router.web_match(&mut req, &mut res).await;

        if resp.is_none() && GLOBAL_HTTP.assets.is_some() {
            (req, res) = GLOBAL_HTTP
                .assets
                .as_mut()
                .unwrap()
                .handle(req, res)
                .unwrap();
        }

        return Ok(teardown(req, res).await.unwrap());
    }
}

#[allow(static_mut_refs)]
async fn handle_web_socket<RW>(
    rw: BufReader<RW>,
    req: &mut Request,
    res: &mut Response,
) -> Result<()>
where
    RW: AsyncPeek + Sync + Send + 'static,
{
    unsafe {
        // TODO: handle unwrap error.
        let (mut handler, req, res) = http1_ws::Handler::new(rw, req, res).await.unwrap();
        let result = GLOBAL_HTTP.router.ws_match(req, res).await;

        if result.is_none() {
            return Ok(());
        }

        let (route, req, res) = result.unwrap();

        return Ok(handler.handle(route, req, res).await.unwrap());
    }
}

async fn http_1_protocol<RW>(mut rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncPeek + Sync + Send + 'static,
{
    let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
    let handle_result = handler.handle().await;

    if handle_result.is_err() {
        return Err(handle_result.err().unwrap());
    }

    let mut req = handle_result.unwrap();
    let mut res = Response::new();

    if req.header("upgrade") == "websocket" {
        (req, res) = setup(req, res).await.unwrap();

        handle_web_socket(rw, &mut req, &mut res).await.unwrap();

        return Ok(());
    }

    (req, res) = handle(req, res).await.unwrap();

    return Ok(handler.write(&mut req, &mut res).await.unwrap());
}

async fn http_2_protocol<RW>(rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncPeek + Sync + Send + 'static,
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

            handler.write(&mut req, &mut res).await.unwrap();
        });
    }

    return Ok(());
}
