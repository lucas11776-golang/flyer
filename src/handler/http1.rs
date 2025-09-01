use std::io::{ErrorKind};
use std::io::{Error as IoError};
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::handler::{parse_request_body, HTTP};
use crate::utils::url::parse_query_params;
use crate::{Values, HTTP as Server};
use crate::request::{Files, Headers, Request};


pub struct Handler {
    server: &'static mut Server,
    addr: SocketAddr,
}


impl <'a>Handler {
    // TODO: refactor handler...
    pub async fn handle<RW>(server: &'a mut Server, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> std::io::Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
            loop {
        // Parse request line
        let mut request_line = String::new();

        let n = rw.read_line(&mut request_line).await?;

        if n == 0 {
            return Ok(());
        }

        if request_line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = request_line.trim_end().split_whitespace().collect();

        if parts.len() != 3 {
            return Err(IoError::new(ErrorKind::InvalidData, "bad request"));
        }

        let method = parts[0].to_string();
        let target = parts[1].to_string();
        let protocol = parts[2].to_string();
        let mut headers = Headers::new();

        loop {
            let mut line = String::new();
            let n = rw.read_line(&mut line).await?;
            
            if n == 0 {
                return Err(IoError::new(ErrorKind::UnexpectedEof, "eof in headers"));
            }
            
            let line_trim = line.trim_end();
            
            if line_trim.is_empty() {
                break;
            }

            if let Some((k, v)) = line_trim.split_once(':') {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }

        let mut body = Vec::new();

        if let Some(te) = headers.get("Transfer-Encoding") {
            if te.eq_ignore_ascii_case("chunked") {
                loop {
                    let mut size_line = String::new();
                    rw.read_line(&mut size_line).await?;
                    let size_str = size_line.trim_end();
                    let size = usize::from_str_radix(size_str, 16)
                        .map_err(|_| IoError::new(ErrorKind::InvalidData, "bad chunk size"))?;
                    if size == 0 {
                        // read trailing CRLF and optional trailers
                        let mut crlf = String::new();
                        rw.read_line(&mut crlf).await?;
                        break;
                    }
                    let mut chunk = vec![0u8; size];
                    tokio::io::AsyncReadExt::read_exact(&mut rw, &mut chunk).await?;
                    body.extend_from_slice(&chunk);
                    // consume CRLF
                    let mut crlf = [0u8; 2];
                    tokio::io::AsyncReadExt::read_exact(&mut rw, &mut crlf).await?;
                }
            }
        } else if let Some(cl) = headers.get("Content-Length") {
            let size = cl.parse::<usize>().map_err(|_| IoError::new(ErrorKind::InvalidData, "bad content-length"))?;
            let mut buf = vec![0u8; size];
            tokio::io::AsyncReadExt::read_exact(&mut rw, &mut buf).await?;
            body = buf;
        }

        let (path, query) = if let Some(i) = target.find('?') {
            (target[..i].to_string(), target[i + 1..].to_string())
        } else {
            (target.clone(), String::new())
        };

        let parameters = parse_query_params(&query);

        let host = headers
            .get("Host")
            .cloned()
            .or_else(|| headers.get("host").cloned())
            .unwrap_or_default();

        let mut req = Request {
            ip: addr.ip().to_string(),
            host: host,
            method: method,
            path: path,
            parameters: parameters,
            protocol: protocol,
            headers: headers,
            body: body,
            values: Values::new(),
            files: Files::new(),
        };

        req.headers.insert("Connection".to_owned(), "keep-alive".to_owned());

        parse_request_body(&mut req).await.unwrap();

        let _ = HTTP::web(server, &mut rw, &mut req).await;
    }
    }
}
