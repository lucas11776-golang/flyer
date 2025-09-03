use std::io::{ErrorKind};
use std::io::{Error as IoError};
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::server::handler::{parse_request_body, Parse};
use crate::server::HTTP1;
use crate::utils::url::parse_query_params;
use crate::utils::Values;
use crate::request::{Files, Headers, Request};
use crate::HTTP;

pub struct Handler { }

impl <'a>Handler {
    // TODO: refactor handler...
    pub async fn handle<RW>(http: &mut HTTP, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> std::io::Result<()> 
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send
    {
        loop {
            // Parse request line
            let mut request_line: String = String::new();
            let n: usize = rw.read_line(&mut request_line).await?;

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

            let method: String = parts[0].to_string();
            let target: String = parts[1].to_string();
            let mut headers: Headers = Headers::new();

            loop {
                let mut line: String = String::new();
                let n: usize = rw.read_line(&mut line).await?;
                
                if n == 0 {
                    return Err(IoError::new(ErrorKind::UnexpectedEof, "eof in headers"));
                }
                
                let line_trim: &str = line.trim_end();
                
                if line_trim.is_empty() {
                    break;
                }

                if let Some((k, v)) = line_trim.split_once(':') {
                    headers.insert(k.trim().to_string(), v.trim().to_string());
                }
            }

            let mut body: Vec<u8> = Vec::new();

            if let Some(te) = headers.get("Transfer-Encoding") {
                if te.eq_ignore_ascii_case("chunked") {
                    loop {
                        let mut size_line = String::new();
                        rw.read_line(&mut size_line).await?;
                        let size_str: &str = size_line.trim_end();
                        let size: usize = usize::from_str_radix(size_str, 16)
                            .map_err(|_| IoError::new(ErrorKind::InvalidData, "bad chunk size"))?;
                        if size == 0 {
                            // read trailing CRLF and optional trailers
                            let mut crlf = String::new();
                            rw.read_line(&mut crlf).await?;
                            break;
                        }
                        let mut chunk: Vec<u8> = vec![0u8; size];
                        tokio::io::AsyncReadExt::read_exact(&mut rw, &mut chunk).await?;
                        body.extend_from_slice(&chunk);
                        // consume CRLF
                        let mut crlf: [u8; 2] = [0u8; 2];
                        tokio::io::AsyncReadExt::read_exact(&mut rw, &mut crlf).await?;
                    }
                }
            } else if let Some(cl) = headers.get("Content-Length") {
                let size = cl.parse::<usize>().map_err(|_| IoError::new(ErrorKind::InvalidData, "bad content-length"))?;
                let mut buffer = vec![0u8; size];
                tokio::io::AsyncReadExt::read_exact(&mut rw, &mut buffer).await?;
                body = buffer;
            }

            let (path, query) = if let Some(i) = target.find('?') {
                (target[..i].to_string(), target[i + 1..].to_string())
            } else {
                (target.clone(), String::new())
            };

            let parameters: Values = parse_query_params(&query);

            let host: String = headers
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
                protocol: HTTP1.to_string(),
                headers: headers,
                body: body,
                values: Values::new(),
                files: Files::new(),
            };

            req.headers.insert("Connection".to_owned(), "keep-alive".to_owned());


            parse_request_body(&mut req).await.unwrap();

            let _ = Parse::web(http, &mut rw, &mut req).await;
        }
    }
}
