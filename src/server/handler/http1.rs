use std::io::{ErrorKind, Result};
use std::io::{Error};
use std::net::SocketAddr;
use std::pin::Pin;

use tokio::io::{
    AsyncBufReadExt,
    AsyncRead,
    AsyncReadExt,
    AsyncWrite,
    AsyncWriteExt,
    BufReader
};

use crate::cookie::Cookies;
use crate::request::form::{Files, Form};
use crate::request::parser::parse_content_type;
use crate::response::parser::parse;
use crate::response::{Response};
use crate::server::protocol::http::APPLICATION;
use crate::utils::url::parse_query_params;
use crate::utils::{Headers, Values};
use crate::request::Request;

const MAX_LINE_LENGTH: usize = 4096;           // 4KB per line (e.g., Request Line)
const MAX_TOTAL_HEADERS_SIZE: usize = 16384;   // 16KB total for all headers

pub(crate) struct Handler<'a, RW> {
    rw: Pin<&'a mut BufReader<RW>>,
    addr: SocketAddr,
}

pub(crate) struct HttpHeader {
    pub method: String,
    pub path: String,
    pub query: Values,
    pub headers: Headers,
}

// TODO: Refactor...
impl<'a, RW> Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(rw: Pin<&'a mut BufReader<RW>>, addr: SocketAddr) -> Self {
        Self { rw, addr }
    }

    pub async unsafe fn handle(&mut self) -> Result<Request> {
        let mut header_data = self.read_http_header().await?;

        let req = Request {
            ip: self.addr.ip().to_string(),
            host: self.get_request_host(&header_data.headers),
            method: header_data.method,
            path: header_data.path,
            parameters: Values::new(),
            query: header_data.query,
            protocol: "HTTP/1.1".to_string(),
            body: self.read_body(&mut header_data.headers).await?,
            headers: header_data.headers,
            form: Form::new(Values::new(), Files::new()),
            session: None,
            cookies: Box::new(Cookies::new(Values::new())),
        };

        return parse_content_type(req).await;
    }

    fn get_request_host(&self, headers: &Headers) -> String {
        headers.get("host").cloned().unwrap_or_default()
    }

    pub async fn write(&mut self, req: &mut Request, res: &mut Response) -> Result<()> {
        let data = parse(res, Some(&mut req.cookies.new_cookie))?;

        self.rw.write_all(data.as_bytes()).await?;
        self.rw.flush().await?;

        Ok(())
    }

    pub async fn read_http_header(&mut self) -> Result<HttpHeader> {
        let mut line = String::new();
        
        // Skip leading empty lines (RFC 9112 allows this for robustness)
        let _ = loop {
            line.clear();

            // Limit line length to prevent OOM
            let read_n = self.rw.as_mut().take(MAX_LINE_LENGTH as u64).read_line(&mut line).await?; // TODO: here is the issue

            if read_n == 0 {
                return Err(Error::new(ErrorKind::ConnectionAborted, "client disconnected"));
            }

            if !line.trim().is_empty() {
                break read_n;
            }
        };

        let parts: Vec<&str> = line.trim_end().split_whitespace().collect();
        if parts.len() != 3 {
            return Err(Error::new(ErrorKind::InvalidData, "invalid request line"));
        }

        let method = parts[0].to_uppercase();
        let target = parts[1].to_string();
        
        // Read headers until the empty line separator (\r\n\r\n)
        let headers = self.read_headers().await?;

        let (path, query) = if let Some(i) = target.find('?') {
            (target[..i].to_string(), target[i + 1..].to_string())
        } else {
            (target.clone(), String::new())
        };

        return Ok(HttpHeader {
            method,
            path,
            query: parse_query_params(&query)?,
            headers,
        });
    }

    async fn read_headers(&mut self) -> Result<Headers> {
        let mut headers = Headers::new();
        let mut total_size = 0;

        loop {
            let mut line = String::new();
            let n = self.rw.as_mut().take(MAX_LINE_LENGTH as u64).read_line(&mut line).await?;
            
            if n == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "connection closed in headers"));
            }
            
            total_size += n;

            if total_size > MAX_TOTAL_HEADERS_SIZE {
                return Err(Error::new(ErrorKind::InvalidData, "headers too large"));
            }

            let trimmed = line.trim_end();

            if trimmed.is_empty() {
                // End of headers
                break; 
            }

            if let Some((k, v)) = trimmed.split_once(':') {
                headers.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }

        return Ok(headers);
    }

    async fn read_body(&mut self, headers: &mut Headers) -> Result<Vec<u8>> {
        // If Transfer-Encoding is present, it MUST take precedence over Content-Length.
        if let Some(te) = headers.get("transfer-encoding") {
            if te.eq_ignore_ascii_case("chunked") {
                return unsafe { self.read_body_transfer_encoding() }.await;
            }
        } 
        
        if let Some(length_str) = headers.get("content-length") {
            let length = length_str.parse::<usize>()
                .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid content-length"))?;
            
            if length > unsafe { APPLICATION.request_max_size } {
                return Err(Error::new(ErrorKind::InvalidData, "body too large"));
            }
            
            return self.read_content_length(length).await;
        }

        Ok(Vec::new())
    }

    async fn read_content_length(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut body = vec![0u8; length];
        self.rw.read_exact(&mut body).await?;
        Ok(body)
    } 

    async unsafe fn read_body_transfer_encoding(&mut self) -> Result<Vec<u8>> {
        let mut body = Vec::new();

        loop {
            let mut size_line = String::new();
            self.rw.read_line(&mut size_line).await?;

            let size_str = size_line.trim_end();
            let size = usize::from_str_radix(size_str, 16)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid chunk size hex"))?;

            if size == 0 {
                let mut trailing = String::new();
                
                // Consume last CRLF
                self.rw.read_line(&mut trailing).await?;

                break;
            }

            if body.len() + size > unsafe { APPLICATION.request_max_size } {
                return Err(Error::new(ErrorKind::InvalidData, "chunked body too large"));
            }

            let mut chunk = vec![0u8; size];
            self.rw.read_exact(&mut chunk).await?;
            body.extend_from_slice(&chunk);

            let mut crlf = [0u8; 2];

             // Consume CRLF after chunk
            self.rw.read_exact(&mut crlf).await?;
        }

        Ok(body)
    }
}