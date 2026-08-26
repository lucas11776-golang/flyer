use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use bytes::{Buf, Bytes, BytesMut};
use h3::server::RequestStream;
use h3_quinn::BidiStream;
use tokio::sync::Mutex as TokioMutex;

use crate::cookies::Cookies;
use crate::request::form::Form;
use crate::request::Request;
use crate::response::{Response, Writer, WriterWrapper};
use crate::server::protocol::UdpHandler;
use crate::server::Server;
use crate::session::Session;
use crate::utils::http::Headers;
use crate::utils::mem::Instance;
use crate::utils::url::parse_query;
use crate::utils::Values;

pub type H3Stream = RequestStream<BidiStream<Bytes>, Bytes>;

pub struct Http3Writer {
    stream: Arc<TokioMutex<H3Stream>>,
    res: Instance<Response>,
    has_sent: Arc<AtomicBool>,
}

impl Http3Writer {
    pub(crate) fn new(
        stream: Arc<TokioMutex<H3Stream>>,
        res: Instance<Response>,
        has_sent: Arc<AtomicBool>,
    ) -> Self {
        Self {
            stream: stream,
            res: res,
            has_sent: has_sent,
        }
    }
}

impl Writer for Http3Writer {
    async fn write(&self, data: Bytes) -> Result<()> {
        let mut stream = self.stream.lock().await;

        if !self.has_sent.swap(true, Ordering::Relaxed) {
            let res =  self.res.as_ref();

            let mut builder = http::Response::builder().status(res.status_code);
            for (k, v) in &res.headers {
                if !k.eq_ignore_ascii_case("content-length") {
                    builder = builder.header(k, v);
                }
            }

            let response_head = builder
                .body(())
                .context("Failed to build HTTP/3 response head")?;

            stream
                .send_response(response_head)
                .await
                .context("Failed to send HTTP/3 response headers")?;
        }

        if !data.is_empty() {
            stream
                .send_data(data)
                .await
                .context("Failed to stream HTTP/3 chunk")?;
        }

        Ok(())
    }
}

pub struct Http3 {
    server: Instance<Server>,
    addr: SocketAddr,
}

impl UdpHandler for Http3 {
    fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self { server, addr }
    }

    async fn handle(
        &mut self,
        mut server: h3::server::Connection<h3_quinn::Connection, Bytes>,
    ) -> Result<()> {
        while let Ok(Some(resolver)) = server.accept().await {
            let server = self.server;
            let addr = self.addr;

            tokio::spawn(async move {
                if let Err(err) = Self::process_request(server, addr, resolver).await {
                    eprintln!("Error handling HTTP/3 request from {addr}: {err:#}");
                }
            });
        }

        Ok(())
    }
}

impl Http3 {
    async fn process_request(
        server: Instance<Server>,
        addr: SocketAddr,
        resolver: h3::server::RequestResolver<h3_quinn::Connection, Bytes>,
    ) -> Result<()> {
        let (request, mut stream) = resolver
            .resolve_request()
            .await
            .context("Failed to resolve HTTP/3 request")?;

        let req = Self::deserialize(server, addr, &request, &mut stream).await?;

        let mut res = Response::new();
        let sent = Arc::new(AtomicBool::new(false));
        let shared_stream = Arc::new(TokioMutex::new(stream));

        let writer = Http3Writer::new(
            shared_stream.clone(),
            Instance::from_mut(&mut res),
            Arc::clone(&sent),
        );

        res.writer = Some(Arc::new(WriterWrapper::new(writer)));

        let server_ref = server.as_mut();
        let (_req, res) = server_ref.on_http(req, res).await;

        let mut stream_lock = shared_stream.lock().await;

        if sent.load(Ordering::Relaxed) {
            stream_lock
                .finish()
                .await
                .context("Failed to finish HTTP/3 stream")?;
            return Ok(());
        }

        Self::write(&mut *stream_lock, res).await
    }

    async fn deserialize(
        server: Instance<Server>,
        addr: SocketAddr,
        request: &http::Request<()>,
        stream: &mut H3Stream,
    ) -> Result<Request> {
        let mut headers = Headers::new();

        for (k, v) in request.headers() {
            if let Ok(val_str) = v.to_str() {
                headers.insert(k.as_str().to_string(), val_str.to_string());
            }
        }

        let host = request
            .uri()
            .authority()
            .map(|a| a.as_str())
            .or_else(|| {
                request
                    .headers()
                    .get(http::header::HOST)
                    .and_then(|h| h.to_str().ok())
            })
            .unwrap_or_default()
            .to_string();

        let body = Self::read_body(stream).await.unwrap_or_default();
        let path = request.uri().path().to_string();
        let queries = parse_query(request.uri().query().unwrap_or(""));

        Ok(Request {
            server: server,
            addr: addr,
            protocol: "HTTP/3.0".into(),
            method: request.method().as_str().to_string(),
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

    async fn read_body(stream: &mut H3Stream) -> Result<Bytes> {
        let mut buf = BytesMut::new();

        while let Some(mut chunk) = stream.recv_data().await? {
            while chunk.has_remaining() {
                let slice = chunk.chunk();
                buf.extend_from_slice(slice);
                chunk.advance(slice.len());
            }
        }

        Ok(buf.freeze())
    }

    pub async fn write(stream: &mut H3Stream, res: Response) -> Result<()> {
        let mut builder = http::Response::builder().status(res.status_code);

        for (k, v) in &res.headers {
            if !k.eq_ignore_ascii_case("content-length") {
                builder = builder.header(k, v);
            }
        }

        builder = builder.header("Content-Length", res.content.len().to_string());

        let response_head = builder
            .body(())
            .context("Failed to build HTTP/3 response")?;

        stream
            .send_response(response_head)
            .await
            .context("Failed to send HTTP/3 headers")?;

        if !res.content.is_empty() {
            stream
                .send_data(res.content)
                .await
                .context("Failed to send HTTP/3 body")?;
        }

        stream
            .finish()
            .await
            .context("Failed to finish HTTP/3 stream")?;

        Ok(())
    }
}