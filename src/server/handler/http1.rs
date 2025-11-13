use std::io::{ErrorKind, Result};
use std::io::{Error as IoError};
use std::net::SocketAddr;
use std::pin::Pin;

use tokio::io::{
    AsyncBufReadExt,
    AsyncRead,
    AsyncWrite,
    AsyncWriteExt,
    BufReader
};

use crate::cookie::Cookies;
use crate::response::{parse, Response};
use crate::server::{HTTP1};
use crate::utils::url::parse_query_params;
use crate::utils::{Values};
use crate::request::{Files, Headers, Request};

pub(crate) struct Handler<'a, RW> {
    rw: Pin<&'a mut BufReader<RW>>,
    addr: SocketAddr,

}

// TODO: user third party HTTP/1.1 parse to handler edge cases...
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

    pub async fn handle<'s>(&'s mut self) -> Option<Result<Request>> {
        let mut request_line: String = String::new();

        // TODO: handle unwrap...
        let n: usize = self.rw.read_line(&mut request_line).await.unwrap();

        if n == 0 {
            return None
        }

        if request_line.trim().is_empty() {
            return None
        }

        let parts: Vec<&str> = request_line.trim_end().split_whitespace().collect();

        if parts.len() != 3 {
            return Some(Err(IoError::new(ErrorKind::InvalidData, "bad request")));
        }

        let method: String = parts[0].to_string();
        let target: String = parts[1].to_string();
        let mut headers: Headers = self.get_headers().await.unwrap();
        let body: Vec<u8> = self.get_body(&mut headers).await.unwrap();

        let (path, query) = if let Some(i) = target.find('?') {
            (target[..i].to_string(), target[i + 1..].to_string())
        } else {
            (target.clone(), String::new())
        };
            
        let host: String = headers
            .get("host")
            .cloned()
            .or_else(|| headers.get("host").cloned())
            .unwrap_or_default();

        let req = Request {
            ip: self.addr.ip().to_string(),
            host: host,
            method: method,
            path: path,
            parameters: Values::new(),
            query: parse_query_params(&query).unwrap(),
            protocol: HTTP1.to_string(),
            headers: headers,
            body: body,
            values: Values::new(),
            files: Files::new(),
            session: None,
            cookies: Cookies::new(Values::new()),
        };

        return Some(Ok(req));
    }

    pub async fn write(&mut self, res: &mut Response) -> Result<()> {
        let _ = self.rw.write(parse(res)?.as_bytes()).await;

        Ok(())
    }

    async fn get_headers(&mut self) -> Result<Headers> {
        let mut headers: Headers = Headers::new();

        loop {
            let mut line: String = String::new();
            let n: usize = self.rw.read_line(&mut line).await?;
            
            if n == 0 {
                return Err(IoError::new(ErrorKind::UnexpectedEof, "eof in headers"));
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

    async fn get_body_transfer_encoding(&mut self) -> Result<Vec<u8>> {
        let mut body: Vec<u8> = Vec::new();

        loop {
            let mut size_line = String::new();

            self.rw.read_line(&mut size_line).await?;

            let size_str: &str = size_line.trim_end();
            let size: usize = usize::from_str_radix(size_str, 16)
                .map_err(|_| IoError::new(ErrorKind::InvalidData, "bad chunk size"))?;

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

    async fn get_body_content_length(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut body = vec![0u8; size];

        tokio::io::AsyncReadExt::read_exact(&mut self.rw, &mut body).await?;

        return Ok(body)
    }

    async fn get_body(&mut self, headers: &mut Headers) -> Result<Vec<u8>> {
        if let Some(te) = headers.get("transfer-encoding") && te.eq_ignore_ascii_case("chunked") {
            return self.get_body_transfer_encoding().await;
        } 
        
        if let Some(cl) = headers.get("content-length") {
            let size = cl.parse::<usize>()
                .map_err(|_| IoError::new(ErrorKind::InvalidData, "bad content-length"))?;

            return self.get_body_content_length(size).await;
        }

        return Ok(vec![]);
    }
}

    