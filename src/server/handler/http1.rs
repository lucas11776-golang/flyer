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
use crate::utils::url::parse_query_params;
use crate::utils::{Headers, Values};
use crate::request::Request;

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

impl <'a, RW>Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    pub fn new(rw: Pin<&'a mut BufReader<RW>>, addr: SocketAddr) -> Self {
        return Self {
            rw: rw,
            addr: addr
        };
    }

    pub async fn handle(&mut self) -> Result<Request> {
        let mut header = self.read_http_header().await?;
        
        let req = Request {
            ip: self.addr.ip().to_string(),
            host: self.get_request_host(&header.headers),
            method: header.method,
            path: header.path,
            parameters: Values::new(),
            query: header.query,
            protocol: "HTTP/1.1".to_string(),
            body: self.read_body(&mut header.headers).await.unwrap(),
            headers: header.headers,
            form: Form::new(Values::new(), Files::new()),
            session: None,
            cookies: Box::new(Cookies::new(Values::new())),
        };

        return Ok(parse_content_type(req).await?);
    }

    fn get_request_host(&mut self, headers: &Headers) -> String {
        return headers.get("host")
            .cloned()
            .or_else(|| headers.get("host").cloned())
            .unwrap_or_default()
    }

    pub async fn write(&mut self, req: &mut Request, res: &mut Response) -> Result<()> {
        #[allow(unused)]
        self.rw
            .write(parse(res, Some(&mut req.cookies.new_cookie))?
            .as_bytes())
            .await;

        Ok(())
    }

    pub async fn read_http_header(&mut self) -> Result<HttpHeader> {
        let mut header_read = String::new();
        let n = self.rw.read_line(&mut header_read).await?;

        if n == 0 || header_read.trim().is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "bad request empty"));
        }

        let header_parts: Vec<&str> = header_read.trim_end().split_whitespace().collect();

        if header_parts.len() != 3 {
            return Err(Error::new(ErrorKind::InvalidData, "bad request structure"));
        }

        let method: String = header_parts[0].to_string();
        let target: String = header_parts[1].to_string();
        let headers: Headers = self.read_headers().await.unwrap();
        let (path, query) = if let Some(i) = target.find('?') {
            (target[..i].to_string(), target[i + 1..].to_string())
        } else {
            (target.clone(), String::new())
        };

        return Ok(HttpHeader {
            method: method.to_uppercase(),
            path: path,
            query: parse_query_params(&query)?,
            headers: headers
        })
    }

    async fn read_headers(&mut self) -> Result<Headers> {
        let mut headers: Headers = Headers::new();

        loop {
            let mut line: String = String::new();
            let n: usize = self.rw.read_line(&mut line).await?;
            
            if n == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "eof in headers"));
            }
            
            let line_trim: &str = line.trim_end();
            
            if line_trim.is_empty() {
                break;
            }

            if let Some((k, v)) = line_trim.split_once(':') {
                headers.insert(k.trim().to_string().to_lowercase(), v.trim().to_string());
            }

        }
        
        return Ok(headers)
    }

    async fn read_body(&mut self, headers: &mut Headers) -> Result<Vec<u8>> {
        let mut body: Vec<u8> = Vec::new();

        if let Some(te) = headers.get("transfer-encoding") && te.eq_ignore_ascii_case("chunked") {
            body.extend(self.read_body_transfer_encoding().await?);
        } 

        if let Some(length) = headers.get("content-length") {
            body.extend(self.read_content_length(length.parse().unwrap()).await?);
        }

        return Ok(body);
    }

    async fn read_content_length(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut body = vec![0u8; length];

        AsyncReadExt::read_exact(&mut self.rw, &mut body).await.unwrap();

        return Ok(body);
    } 

    async fn read_body_transfer_encoding(&mut self) -> Result<Vec<u8>> {
        let mut body: Vec<u8> = Vec::new();

        loop {
            let mut size_line = String::new();

            self.rw.read_line(&mut size_line).await?;

            let size_str: &str = size_line.trim_end();
            let size: usize = usize::from_str_radix(size_str, 16)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "bad chunk size"))?;

            if size == 0 {
                // read trailing CRLF and optional trailers
                let mut crlf = String::new();
                
                self.rw.read_line(&mut crlf).await?;

                break;
            }

            let mut chunk: Vec<u8> = vec![0u8; size];

            tokio::io::AsyncReadExt::read_exact(&mut self.rw, &mut chunk).await?;

            body.extend_from_slice(&chunk);

            // consume CRLF
            let mut crlf: [u8; 2] = [0u8; 2];

            tokio::io::AsyncReadExt::read_exact(&mut self.rw, &mut crlf).await?;
        }

        return Ok(body);
    }
}

    