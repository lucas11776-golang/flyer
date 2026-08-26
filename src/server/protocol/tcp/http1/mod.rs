use std::io::{Error, ErrorKind, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::request::form::Form;
use crate::request::Request;
use crate::response::{WriterWrapper, Response, Writer};
use crate::server::protocol::TcpHandler;
use crate::server::Server;
use crate::server::protocol::tcp::http1::ws::Ws;
use crate::utils::http::Headers;
use crate::utils::mem::Instance;
use crate::utils::url::parse_query;
use crate::utils::Values;

pub mod ws;

const MAX_HEADER_LENGTH: usize = 8192; // 8KB standard cap

pub struct Http1 {
    server: Instance<Server>,
    addr: SocketAddr,
}

pub struct Http1Writer<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    rw: Instance<BufReader<RW>>,
    res: Instance<Response>,
    has_sent: Arc<AtomicBool>,
}

impl<RW> Http1Writer<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    pub fn new(
        rw: Instance<BufReader<RW>>,
        res: Instance<Response>,
        has_sent: Arc<AtomicBool>,
    ) -> Self {
        Self { rw, res, has_sent }
    }
}

impl<RW> Writer for Http1Writer<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    async fn write(&self, data: Bytes) -> Result<()> {
        let rw =  self.rw.as_mut();
        let res =  self.res.as_mut();

        if !self.has_sent.swap(true, Ordering::Relaxed) {
            Http1::write_header(rw, res).await?;
        }

        rw
            .write_all(&data)
            .await
            .map_err(Into::into)
    }
}

impl TcpHandler for Http1 {
    fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self { server, addr }
    }

    async fn handle<RW>(&mut self, mut rw: BufReader<RW>) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let req = match self.deserialize(&mut rw).await {
            Ok(req) => req,
            Err(_) => return Ok(()),
        };

        if req.header("upgrade").eq_ignore_ascii_case("websocket") {
            return Ws::new(self.server, self.addr).handle(rw, req).await;
        }

        let mut res = Response::new();
        let sent = Arc::new(AtomicBool::new(false));

        let writer = Http1Writer::new(
            Instance::from_mut(&mut rw),
            Instance::from_mut(&mut res),
            Arc::clone(&sent),
        );

        res.writer = Some(Arc::new(WriterWrapper::new(writer)));

        let server =  self.server.as_mut();
        
        let (_req, res) = server.on_http(req, res).await;

        if sent.load(Ordering::Relaxed) {
            return Ok(());
        }

        let content_length = res.content.len();

        let mut res = res.set_header("Content-Length", content_length.to_string());

        Self::write_response(&mut rw, &mut res).await
    }
}

impl Http1 {
    async fn deserialize<RW>(&mut self, rw: &mut BufReader<RW>) -> Result<Request>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    {
        let mut buffer = BytesMut::with_capacity(MAX_HEADER_LENGTH);

        let header_size = loop {
            let n = rw.read_buf(&mut buffer).await?;
            if n == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "connection closed").into());
            }

            let mut headers_ptr = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers_ptr);

            match req.parse(&buffer) {
                Ok(httparse::Status::Complete(size)) => break size,
                Ok(httparse::Status::Partial) => {
                    if buffer.len() >= MAX_HEADER_LENGTH {
                        return Err(
                            Error::new(ErrorKind::InvalidData, "HTTP header limit exceeded").into(),
                        );
                    }
                }
                Err(e) => return Err(Error::new(ErrorKind::InvalidData, e).into()),
            }
        };

        let header_bytes = buffer.split_to(header_size);
        let leftover_body = buffer;

        let mut headers_ptr = [httparse::EMPTY_HEADER; 64];
        let mut parsed_req = httparse::Request::new(&mut headers_ptr);

        parsed_req
            .parse(&header_bytes)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        let mut headers = Headers::new();
        let mut content_length: u64 = 0;
        let mut is_chunked = false;

        for h in parsed_req.headers.iter().filter(|h| !h.name.is_empty()) {
            let val_str = std::str::from_utf8(h.value).unwrap_or("").trim();
            let name = h.name;

            if name.eq_ignore_ascii_case("content-length") {
                content_length = val_str.parse().unwrap_or(0);
            } else if name.eq_ignore_ascii_case("transfer-encoding") && val_str.contains("chunked") {
                is_chunked = true;
            }

            headers.insert(name.to_ascii_lowercase(), val_str.to_string());
        }

        let body = if is_chunked {
            Self::read_chunked_body(rw, leftover_body).await?
        } else {
            Self::read_fixed_body(rw, leftover_body, content_length).await?
        };

        let raw_url = parsed_req.path.unwrap_or("");
        let (path, queries) = match raw_url.find('?') {
            Some(i) => (&raw_url[..i], parse_query(&raw_url[i + 1..])),
            None => (raw_url, Values::new()),
        };

        let host = headers.get("host").cloned().unwrap_or_default();

        Ok(Request {
            server: self.server,
            addr: self.addr,
            protocol: "HTTP/1.1".to_string(),
            method: parsed_req.method.unwrap_or("GET").to_string(),
            path: path.to_string(),
            queries: queries,
            host: host,
            headers: headers,
            parameters: Values::new(),
            cookies: Default::default(),
            session: Default::default(),
            body: body.into(),
            form: Form::default(),
        })
    }

    async fn read_fixed_body<RW>(
        rw: &mut BufReader<RW>,
        mut leftover: BytesMut,
        content_length: u64,
    ) -> Result<Vec<u8>>
    where
        RW: AsyncRead + Unpin + Send + Sync,
    {
        if content_length == 0 {
            return Ok(Vec::new());
        }

        let target_len = content_length as usize;

        if leftover.len() >= target_len {
            return Ok(leftover.split_to(target_len).to_vec());
        }

        let additional_needed = target_len - leftover.len();
        leftover.reserve(additional_needed);

        while leftover.len() < target_len {
            if rw.read_buf(&mut leftover).await? == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Truncated HTTP body").into());
            }
        }

        Ok(leftover.split_to(target_len).to_vec())
    }

    async fn read_chunked_body<RW>(
        rw: &mut BufReader<RW>,
        mut buf: BytesMut,
    ) -> Result<Vec<u8>>
    where
        RW: AsyncRead + Unpin + Send + Sync,
    {
        let mut body = Vec::new();
        let mut search_idx = 0;

        loop {
            let line_end = loop {
                if let Some(pos) = buf[search_idx..].windows(2).position(|w| w == b"\r\n") {
                    break search_idx + pos;
                }
                search_idx = buf.len().saturating_sub(1);
                if rw.read_buf(&mut buf).await? == 0 {
                    return Err(
                        Error::new(ErrorKind::UnexpectedEof, "Truncated chunk size").into(),
                    );
                }
            };

            let size_bytes = buf.split_to(line_end);
            buf.advance(2);
            search_idx = 0;

            let size_str = std::str::from_utf8(&size_bytes)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in chunk size"))?;
            let hex_str = size_str.split(';').next().unwrap_or("").trim();
            let chunk_size = usize::from_str_radix(hex_str, 16)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid hex chunk size"))?;

            if chunk_size == 0 {
                while buf.len() < 2 {
                    if rw.read_buf(&mut buf).await? == 0 {
                        break;
                    }
                }
                break;
            }

            let total_needed = chunk_size + 2;
            while buf.len() < total_needed {
                if rw.read_buf(&mut buf).await? == 0 {
                    return Err(
                        Error::new(ErrorKind::UnexpectedEof, "Truncated chunk body").into(),
                    );
                }
            }

            body.extend_from_slice(&buf[..chunk_size]);
            buf.advance(total_needed);
        }

        Ok(body)
    }

    fn format_headers(res: &Response, buf: &mut BytesMut) {
        use std::fmt::Write;

        let status_code = res.status_code;
        let status_text = http::StatusCode::from_u16(status_code)
            .map(|s| s.canonical_reason().unwrap_or("OK"))
            .unwrap_or("OK");

        let _ = write!(buf, "HTTP/1.1 {} {}\r\n", status_code, status_text);

        for (k, v) in &res.headers {
            let _ = write!(buf, "{}: {}\r\n", k, v);
        }

        let _ = write!(buf, "\r\n");
    }

    async fn write_header<RW>(rw: &mut BufReader<RW>, res: &mut Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let mut buffer = BytesMut::with_capacity(256 + res.headers.len() * 32);

        Self::format_headers(res, &mut buffer);

        rw
            .write_all(&buffer)
            .await
            .unwrap();

        rw
            .flush()
            .await
            .map_err(Into::into)
    }

    // TODO: when request is large send as chunks.
    async fn write_response<RW>(rw: &mut BufReader<RW>, res: &mut Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let mut buffer = BytesMut::with_capacity(256 + res.headers.len() * 32 + res.content.len());

        Self::format_headers(res, &mut buffer);

        if res.content.len() <= 16384 {
            buffer.extend_from_slice(&res.content);
            rw.write_all(&buffer).await?;
        } else {
            rw.write_all(&buffer).await?;
            rw.write_all(&res.content).await?;
        }

        rw
            .flush()
            .await
            .map_err(Into::into)
    }
}