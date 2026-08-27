use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use bytes::{Bytes, BytesMut};
use h2::server::{self, SendResponse};
use h2::{RecvStream, SendStream};
use tokio::io::{AsyncRead, AsyncWrite, BufReader};
use tokio::sync::Mutex as TokioMutex;

use crate::cookies::Cookies;
use crate::request::form::Form;
use crate::request::Request;
use crate::response::{Response, Writer, WriterWrapper};
use crate::server::protocol::TcpHandler;
use crate::server::Server;
use crate::session::Session;
use crate::utils::http::Headers;
use crate::utils::mem::Instance;
use crate::utils::url::parse_query;
use crate::utils::Values;

pub(crate) const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub enum Http2StreamState {
    Unsent(SendResponse<Bytes>),
    Sent(SendStream<Bytes>),
    Closed,
}

pub struct Http2Writer {
    state: Arc<TokioMutex<Http2StreamState>>,
    res: Instance<Response>,
    has_sent: Arc<AtomicBool>,
}

impl Http2Writer {
    pub fn new(
        state: Arc<TokioMutex<Http2StreamState>>,
        res: Instance<Response>,
        has_sent: Arc<AtomicBool>,
    ) -> Self {
        Self {
            state,
            res,
            has_sent,
        }
    }
}

impl Writer for Http2Writer {
    async fn write(&self, data: Bytes) -> Result<()> {
        let mut lock = self.state.lock().await;

        if !self.has_sent.swap(true, Ordering::Relaxed) {
            let res =  self.res.as_ref();

            if let Http2StreamState::Unsent(mut send_response) =
                std::mem::replace(&mut *lock, Http2StreamState::Closed)
            {
                let mut builder = http::Response::builder().status(res.status_code);
                for (k, v) in &res.headers {
                    builder = builder.header(k, v);
                }

                let response_head = builder.body(()).context("Failed to build HTTP response head")?;
                let send_stream = send_response
                    .send_response(response_head, false)
                    .context("Failed to send HTTP/2 headers")?;

                *lock = Http2StreamState::Sent(send_stream);
            }
        }

        if let Http2StreamState::Sent(ref mut send_stream) = *lock {
            Http2::send_data_chunked(send_stream, data, false).await?;
        }

        Ok(())
    }
}

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
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let mut connection = server::handshake(rw)
            .await
            .map_err(|err| anyhow::anyhow!("H2 Handshake error: {err}"))?;

        while let Some(result) = connection.accept().await {
            match result {
                Ok((request, respond)) => {
                    let server = self.server;
                    let addr = self.addr;

                    tokio::spawn(async move {
                        if let Err(err) = Self::process_request(server, addr, request, respond).await {
                            eprintln!("Error handling HTTP/2 request from {addr}: {err:#}");
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
        respond: SendResponse<Bytes>,
    ) -> Result<()> {
        let req = Self::deserialize(server, addr, request).await?;
        let mut res = Response::new();
        let sent = Arc::new(AtomicBool::new(false));

        let stream_state = Arc::new(TokioMutex::new(Http2StreamState::Unsent(respond)));

        let writer = Http2Writer::new(
            stream_state.clone(),
            Instance::from_mut(&mut res),
            Arc::clone(&sent),
        );

        res.writer = Some(Arc::new(WriterWrapper::new(writer)));

        let server_ref =  server.as_mut();
        
        let (_req, res) = server_ref.on_http(req, res).await;

        let mut lock = stream_state.lock().await;

        if sent.load(Ordering::Relaxed) {
            if let Http2StreamState::Sent(ref mut send_stream) = *lock {
                let _ = send_stream.send_data(Bytes::new(), true);
            }
            return Ok(());
        }

        if let Http2StreamState::Unsent(send_response) =
            std::mem::replace(&mut *lock, Http2StreamState::Closed)
        {
            Self::write(send_response, res).await?;
        }

        Ok(())
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
            let len = chunk.len();

            buf.extend_from_slice(&chunk);

            let _ = stream.flow_control().release_capacity(len);
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
            Self::send_data_chunked(&mut send_stream, res.content, true)
                .await
                .unwrap();
        }

        Ok(())
    }

    pub(crate) async fn send_data_chunked(send_stream: &mut SendStream<Bytes>, mut data: Bytes, end_stream: bool) -> Result<()> {
        if data.is_empty() {
            if end_stream {
                send_stream
                    .send_data(Bytes::new(), true)
                    .context("Failed to close H2 stream")?;
            }
            return Ok(());
        }

        while !data.is_empty() {
            send_stream.reserve_capacity(data.len());
            let cap = std::future::poll_fn(|cx| send_stream.poll_capacity(cx))
                .await
                .ok_or_else(|| anyhow::anyhow!("H2 stream closed by peer"))??;

            if cap == 0 {
                continue;
            }

            let chunk_size = data.len().min(cap);
            let chunk = data.split_to(chunk_size);
            let is_last = end_stream && data.is_empty();

            send_stream
                .send_data(chunk, is_last)
                .context("Failed to send chunk data over H2 stream")?;
        }

        Ok(())
    }
}