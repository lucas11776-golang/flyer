use std::io::{ErrorKind, Result};
use std::io::{Error as IoError};
use std::net::SocketAddr;
use std::pin::Pin;

use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use openssl::sha::{Sha1};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Role::Server;
use tungstenite::{Message};

use crate::response::{new_response, parse};
use crate::server::handler::ws::SEC_WEB_SOCKET_ACCEPT_STATIC;
use crate::server::handler::{parse_request_body, ws, RequestHandler};
use crate::server::HTTP1;
use crate::utils::url::parse_query_params;
use crate::utils::{Values};
use crate::request::{Files, Headers, Request};
use crate::ws::{Event, Reason, Ws};
use crate::HTTP;

pub struct Handler { }

impl <'a>Handler {
    pub fn new() -> Self {
        return Self{};
    }

    pub async fn handle<RW>(&mut self, http: &'a mut HTTP, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()> 
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut request_line: String = String::new();
        let n: usize = rw.read_line(&mut request_line).await?;

        if n == 0 {
            return Ok(());
        }

        if request_line.trim().is_empty() {
            return Ok(());
        }

        let parts: Vec<&str> = request_line.trim_end().split_whitespace().collect();

        if parts.len() != 3 {
            return Err(IoError::new(ErrorKind::InvalidData, "bad request"));
        }

        let method: String = parts[0].to_string();
        let target: String = parts[1].to_string();
        let mut headers: Headers = self.get_headers(&mut rw).await?;
        let body: Vec<u8> = self.get_body(&mut rw, &mut headers).await?;

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
            ip: addr.ip().to_string(),
            host: host,
            method: method,
            path: path,
            parameters: Values::new(),
            query: parse_query_params(&query)?,
            protocol: HTTP1.to_string(),
            headers: headers,
            body: body,
            values: Values::new(),
            files: Files::new(),
        };

        if req.headers.get("upgrade").is_some() && req.headers.get("upgrade").unwrap().to_lowercase() == "websocket" {
            return ws::Handler::new(http, rw, req).handle().await;
        }

        return Ok(self.handle_web_request(http, rw, req).await.unwrap());
    }

    async fn handle_web_request<R>(&mut self, http: &mut HTTP, mut sender: Pin<&mut BufReader<R>>, mut req: Request) -> Result<()>
    where
        R: AsyncRead + AsyncWrite + Unpin + Send
    {
        req.headers.insert("Connection".to_owned(), "keep-alive".to_owned());

        let req = parse_request_body(&mut req).await.unwrap();
        let res = &mut new_response();

        let res = RequestHandler::web(http, req, res).await?;
        let _ = sender.write(parse(res)?.as_bytes()).await;

        Ok(())
    }

    async fn get_headers<RW>(&mut self, sender: &'a mut Pin<&mut BufReader<RW>>) -> Result<Headers>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        let mut headers: Headers = Headers::new();

        loop {
            let mut line: String = String::new();
            let n: usize = sender.read_line(&mut line).await?;
            
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


    async fn get_body_transfer_encoding<RW>(&mut self, mut sender: &'a mut Pin<&mut BufReader<RW>>) -> Result<Vec<u8>>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        let mut body: Vec<u8> = Vec::new();

        loop {
            let mut size_line = String::new();

            sender.read_line(&mut size_line).await?;

            let size_str: &str = size_line.trim_end();
            let size: usize = usize::from_str_radix(size_str, 16)
                .map_err(|_| IoError::new(ErrorKind::InvalidData, "bad chunk size"))?;

            if size == 0 {
                // read trailing CRLF and optional trailers
                let mut crlf = String::new();
                
                sender.read_line(&mut crlf).await?;

                break;
            }

            let mut chunk: Vec<u8> = vec![0u8; size];

            tokio::io::AsyncReadExt::read_exact(&mut sender, &mut chunk).await?;

            body.extend_from_slice(&chunk);

            // consume CRLF
            let mut crlf: [u8; 2] = [0u8; 2];

            tokio::io::AsyncReadExt::read_exact(&mut sender, &mut crlf).await?;
        }

        return Ok(body);
    }


    async fn get_body_content_length<RW>(&mut self, mut sender: &'a mut Pin<&mut BufReader<RW>>, size: usize) -> Result<Vec<u8>>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync

    {
        let mut body = vec![0u8; size];

        tokio::io::AsyncReadExt::read_exact(&mut sender, &mut body).await?;

        return Ok(body)
    }

    async fn get_body<RW>(&mut self, sender: &'a mut Pin<&mut BufReader<RW>>, headers: &mut Headers) -> Result<Vec<u8>>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        if let Some(te) = headers.get("transfer-encoding") && te.eq_ignore_ascii_case("chunked") {
            return self.get_body_transfer_encoding(sender).await;
        } 
        
        if let Some(cl) = headers.get("content-length") {
            let size = cl.parse::<usize>()
                .map_err(|_| IoError::new(ErrorKind::InvalidData, "bad content-length"))?;

            return self.get_body_content_length(sender, size).await;
        }

        return Ok(vec![]);
    }
}

    
