use std::io::ErrorKind;
use std::net::SocketAddr;

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader}, net::TcpListener};
use tokio_rustls::TlsAcceptor;

use crate::{server::{Server, transport::Protocol}, utils::server::get_tls_acceptor, warn};

pub async fn listen(server_ptr: usize) {
    unsafe {
        listener(
            server_ptr as usize,
            TcpListener::bind((*(server_ptr as *const Server)).address()).await.unwrap(),
            &(*(server_ptr as *const Server)).server_config.clone().map(|config| get_tls_acceptor(config.clone()).unwrap()) as *const Option<TlsAcceptor> as usize
        ).await;
    }
}

async fn listener(server_ptr: usize, listener: TcpListener, tls_ptr: usize) {
    unsafe  {
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(async move {
                match &*(tls_ptr as *const Option<TlsAcceptor>) {
                    Some(acceptor) => {
                        match acceptor.accept(stream).await {
                            Ok(rw) => connection(server_ptr, BufReader::new(rw), addr).await,
                            Err(err) => warn!("TLS connection error"; "error" => err),
                        }
                    },
                    None => {
                        connection(server_ptr, BufReader::new(stream), addr).await
                    },
                }
            });
        }
    }
}

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

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

async fn connection<RW>(server_ptr: usize, mut rw: BufReader<RW>, addr: SocketAddr)
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
        // let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
        // let handle_result = handler.handle().await;

        // if handle_result.is_err() {
        //     return Err(handle_result.err().unwrap());
        // }

        // let req = handle_result.unwrap();
        // let res = Response::new();

        // if req.header("upgrade").to_lowercase() == "websocket" {
        //     let (mut req, mut res) = RequestHandler::new().setup(req, res).await.unwrap();

        //     handle_web_socket(rw, &mut req, &mut res).await.unwrap();

        //     return Ok(());
        // }

        // let (mut req, mut res) = APPLICATION.on_request(req, res).await.unwrap();

        // handler.write(&mut req, &mut res).await.unwrap();

        // drop(rw);

        return Ok(());
    }
}

#[allow(static_mut_refs)]
async fn http_2_protocol<RW>(rw: BufReader<RW>, addr: SocketAddr) -> Result<()>
where
    RW: AsyncRead + AsyncWrite + Unpin  + Sync + Send + 'static,
{
    unsafe {


        // let mut conn = server::handshake(rw).await.unwrap();

        // while let Some(result) = conn.accept().await {
        //     if result.is_err() {
        //         continue;
        //     }

        //     tokio::spawn(async move {
        //         let (request, send) = result.unwrap();
        //         let mut handler = http2::Handler::new(addr, send);
        //         let req = handler.handle(request).await.unwrap();
        //         let (mut req, mut res) = APPLICATION.on_request(req, Response::new()).await.unwrap();

        //         handler.write(&mut req, &mut res).await.unwrap();
        //     });
        // }

        return Ok(());
    }
}
