use std::net::SocketAddr;

use anyhow::{Context, Result};
use bytes::{Bytes, BytesMut};
use h2::server::{self, SendResponse};
use h2::RecvStream;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::cookies::Cookies;
use crate::request::form::Form;
use crate::request::Request;
use crate::response::Response;
use crate::server::protocol::TcpHandler;
use crate::server::Server;
use crate::session::Session;
use crate::utils::http::Headers;
use crate::utils::mem::Instance;
use crate::utils::url::parse_query;
use crate::utils::Values;

pub struct Http2 {
    server: Instance<Server>,
    addr: SocketAddr,
}

impl TcpHandler for Http2 {
    fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self { server, addr }
    }

    async fn handle<RW>(&mut self, rw: BufReader<RW>) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    {
        let mut connection = server::handshake(rw)
            .await
            .map_err(|err| anyhow::anyhow!("H2 Handshake error: {err}"))?;

        while let Some(result) = connection.accept().await {
            match result {
                Ok((request, respond)) => {
                    let server = self.server.clone();
                    let addr = self.addr;

                    tokio::spawn(async move {
                        if let Err(err) = Self::process_request(server, addr, request, respond).await {
                            eprintln!("Error handling HTTP/2 request from {addr}: {err}");
                        }
                    });
                }
                Err(err) => {
                    eprintln!("HTTP/2 stream accept error: {err}");
                }
            }
        }

        Ok(())
    }
}

impl Http2 {
    async fn process_request(
        server: Instance<Server>,
        addr: SocketAddr,
        request: http::Request<RecvStream>,
        response: SendResponse<Bytes>,
    ) -> Result<()> {
        let req = Self::deserialize(server.clone(), addr, request).await?;
        let (_, res) = server.as_mut().on_http(req, Response::new()).await;
        Self::write(response, res).await
    }

    async fn deserialize(
        server: Instance<Server>,
        addr: SocketAddr,
        request: http::Request<RecvStream>,
    ) -> Result<Request> {
        let (parts, body_stream) = request.into_parts();
        
        let body = Self::read_full_body(body_stream).await?;
        let host = parts
            .uri
            .authority()
            .map(|a| a.as_str())
            .or_else(|| {
                parts
                    .headers
                    .get(http::header::HOST)
                    .and_then(|h| h.to_str().ok())
            })
            .unwrap_or_default()
            .to_string();

        let mut headers = Headers::new();

        for (k, v) in parts.headers.iter() {
            if let Ok(val_str) = v.to_str() {
                headers.insert(k.as_str().to_string(), val_str.to_string());
            }
        }

        let path = parts.uri.path().to_string();
        let queries = parse_query(parts.uri.query().unwrap_or(""));

        Ok(Request {
            server: server,
            addr: addr,
            protocol: "HTTP/2.0".into(),
            method: parts.method.to_string(),
            path: path,
            queries: queries,
            host: host,
            headers: headers,
            cookies: Cookies::new(),
            session: Session::new(),
            body: body,
            parameters: Values::new(),
            form: Form::new(Default::default(), Default::default()),
        })
    }

    async fn read_full_body(mut stream: RecvStream) -> Result<Bytes> {
        let mut buf = BytesMut::new();

        while let Some(chunk) = stream.data().await {
            let chunk = chunk.context("Error reading H2 request body stream")?;
            let _ = stream.flow_control().release_capacity(chunk.len());

            buf.extend_from_slice(&chunk);
        }

        Ok(buf.freeze())
    }

    pub async fn write(mut send_response: SendResponse<Bytes>, res: Response) -> Result<()> {
        let mut builder = http::Response::builder().status(res.status_code);

        for (k, v) in &res.headers {
            builder = builder.header(k, v);
        }

        let is_empty = res.content.is_empty();
        let response_head = builder.body(()).context("Failed to build HTTP response")?;
        let mut send_stream = send_response
            .send_response(response_head, is_empty)
            .context("Failed to send response headers")?;

        if !is_empty {
            send_stream
                .send_data(res.content, true)
                .context("Failed to send response body")?;
        }

        Ok(())
    }
}