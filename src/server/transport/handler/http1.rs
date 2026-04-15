use std::io::Cursor;
use std::io::{ErrorKind};
use std::io::{Error};
use std::net::SocketAddr;
use std::pin::Pin;

use anyhow::Result;
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::cookies::Cookies;
use crate::request::form::{Files, Form};
use crate::response::{Response};
use crate::server::helpers::parse::http_1_parse;
use crate::utils::url::parse_query_params;
use crate::utils::{Headers, Values};
use crate::request::Request;

const MAX_LINE_LENGTH: usize = 4096; // 4KB per line (e.g., Request Line)

#[allow(dead_code)]
pub(crate) struct Handler<'a, RW> {
    server_ptr: usize,
    rw: Pin<&'a mut BufReader<RW>>,
    addr: SocketAddr,
}

impl<'a, RW> Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(ptr: usize, rw: Pin<&'a mut BufReader<RW>>, addr: SocketAddr) -> Self {
        return Self {
            server_ptr: ptr,
            rw: rw,
            addr: addr
        };
    }

    pub async unsafe fn handle(&mut self) -> Result<Request> {
        let mut buffer = BytesMut::with_capacity(MAX_LINE_LENGTH);

        let header_size = loop {
            let n = self.rw.as_mut().read_buf(&mut buffer).await
                .map_err(|e| Error::new(ErrorKind::Other, e))?;
            
            if n == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "connection closed").into());
            }

            let mut headers_ptr = [httparse::EMPTY_HEADER; 64];
            let mut req = httparse::Request::new(&mut headers_ptr);

            // TODO: unhandled error...
            if let httparse::Status::Complete(size) = req.parse(&buffer).unwrap() {
                break size;
            }
        };

        let header_data = buffer.split_to(header_size);
        let leftover_body = buffer;

        let mut headers_ptr = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers_ptr);
        req.parse(&header_data).unwrap();

        let is_chunked = req.headers.iter()
            .any(|h| h.name.eq_ignore_ascii_case("Transfer-Encoding") 
                && std::str::from_utf8(h.value).unwrap_or("").contains("chunked"));

        let mut body = Vec::new();

        if is_chunked {
            let mut reader = Cursor::new(leftover_body).chain(self.rw.as_mut());
            
            loop {
                let mut size_buf = Vec::new();
                let mut found_newline = false;
                while !found_newline {
                    let b = reader.read_u8().await.map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
                    size_buf.push(b);
                    if size_buf.ends_with(b"\n") {
                        found_newline = true;
                    }
                }
                
                let line_str = String::from_utf8_lossy(&size_buf);
                let chunk_size = u64::from_str_radix(line_str.trim().split(';').next().unwrap_or("0"), 16)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid chunk size"))?;

                if chunk_size == 0 {
                    let mut trailer = [0u8; 2];
                    let _ = reader.read_exact(&mut trailer).await;
                    break;
                }

                let mut chunk_data = vec![0u8; chunk_size as usize];
                reader.read_exact(&mut chunk_data).await.map_err(|e| Error::from(e))?;
                body.extend(chunk_data);

                let mut crlf = [0u8; 2];
                reader.read_exact(&mut crlf).await.map_err(|e| Error::from(e))?;
            }
        } else {
            let content_length = req.headers.iter()
                .find(|h| h.name.eq_ignore_ascii_case("Content-Length"))
                .and_then(|h| std::str::from_utf8(h.value).ok()?.parse::<u64>().ok())
                .unwrap_or(0);
            
            body.extend_from_slice(&leftover_body);
            let remaining = content_length.saturating_sub(leftover_body.len() as u64);

            if remaining > 0 {
                let mut limited = self.rw.as_mut().take(remaining);
                limited.read_to_end(&mut body).await.map_err(|e| Error::from(e))?;
            }
        }

        let mut headers = Headers::new();

        for h in req.headers.iter().filter(|h| !h.name.is_empty()) {
            headers.insert(h.name.to_lowercase(), String::from_utf8_lossy(h.value).to_string());
        }

        let url = req.path.unwrap_or("");
        let (path, query) = if let Some(i) = url.find('?') {
            (&url[..i], parse_query_params(&url[i + 1..].to_string()))
        } else {
            (url, Values::new())
        };

        Ok(Request {
            cookies: Box::new(Cookies::new(Values::new())),
            session: None,
            ip: self.addr.ip().to_string(),
            host: headers.get("host").cloned().unwrap_or_default(),
            method: req.method.unwrap_or("GET").to_string(),
            path: path.to_string(),
            query,
            parameters: Values::new(),
            protocol: "HTTP/1.1".to_string(),
            headers,
            body,
            form: Form::new(Values::new(), Files::new()),
        })
    }

    pub async fn write(&mut self, req: &mut Request, res: &mut Response) -> Result<()> {
        self.rw.write_all(&http_1_parse(res, Some(&mut req.cookies.new_cookie))).await.unwrap();
        self.rw.flush().await.unwrap();
        Ok(())
    }
}



