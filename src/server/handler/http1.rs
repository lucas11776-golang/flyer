use std::io::{ErrorKind, Result};
use std::io::{Error as IoError};
use std::net::SocketAddr;
use std::pin::Pin;

use futures_util::{StreamExt};
use openssl::sha::{Sha1};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Role::Server;
use tungstenite::{Message};

use crate::response::{new_response, parse};
use crate::server::handler::{parse_request_body, RequestHandler};
use crate::server::HTTP1;
use crate::utils::url::parse_query_params;
use crate::utils::{Pointer, Values};
use crate::request::{Files, Headers, Request};
use crate::ws::{Reason, Ws, SEC_WEB_SOCKET_ACCEPT_STATIC};
use crate::HTTP;

pub struct Handler { }

impl <'a>Handler {
    pub async fn handle<RW>(http: &mut HTTP, mut sender: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> std::io::Result<()> 
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        loop {
            let mut request_line: String = String::new();
            let n: usize = sender.read_line(&mut request_line).await?;

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

            let mut body: Vec<u8> = Vec::new();

            if let Some(te) = headers.get("transfer-encoding") {
                if te.eq_ignore_ascii_case("chunked") {
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
                }
            } else if let Some(cl) = headers.get("content-length") {
                let size = cl.parse::<usize>().map_err(|_| IoError::new(ErrorKind::InvalidData, "bad content-length"))?;
                let mut buffer = vec![0u8; size];
                tokio::io::AsyncReadExt::read_exact(&mut sender, &mut buffer).await?;
                body = buffer;
            }

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

            return Ok(
                match req.headers.get("upgrade").cloned() {
                    Some(upgrade) => {
                        if upgrade == "websocket".to_string() {
                            return Ok(Handler::handle_ws_request(http, sender, req).await?);
                        }

                        Handler::handle_web_request(http, sender, req).await?;
                    },
                    None => Handler::handle_web_request(http, sender, req).await?,
                }
            )
        }
    }

    async fn handle_web_request<R>(http: &mut HTTP, mut sender: Pin<&mut BufReader<R>>, mut req: Request) -> Result<()>
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

    fn generate_accept_key(key: String) -> String {
        let mut hasher = Sha1::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());

        let result = hasher.finish();
        
        return base64::encode(&result)
    }

    async fn handle_ws_request<'b, R>(http: &mut HTTP, mut sender: Pin<&mut BufReader<R>>, mut req: Request) -> Result<()>
    where
        R: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        let sec_websocket_key = req.header("sec-websocket-key");
        let mut resp = new_response();
        
        resp.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), Handler::generate_accept_key(sec_websocket_key));

        sender.write(parse(&mut resp).unwrap().as_bytes()).await.unwrap();

        let mut stream = WebSocketStream::from_raw_socket(sender, Server, None).await;
        let mut res = new_response();
        let mut  ws_pointer = Pointer::new(Ws::new());

        res.ws = Some(ws_pointer.point());

        let ws = http.router().router.match_ws_routes(&mut req, &mut res).await;

        if ws.is_none() {
            return Ok(());
        }

        let ws = ws.unwrap();

        while let Some(msg) = stream.next().await {
            match msg.unwrap() {
                Message::Text(data) => {
                    
                    if ws.message.is_some() {
                        ws.message.as_ref().unwrap()(ws_pointer.point(), data.as_bytes().to_vec()).await;
                    }
                },
                Message::Binary(bytes) => {
                    if ws.message.is_some() {
                        ws.message.as_ref().unwrap()(ws_pointer.point(), bytes.to_vec()).await;
                    }
                },
                Message::Ping(bytes) => {
                    if ws.ping.is_some() {
                        ws.ping.as_ref().unwrap()(ws_pointer.point(), bytes.to_vec()).await;
                    }
                },
                Message::Pong(bytes) => {
                    if ws.pong.is_some() {
                        ws.pong.as_ref().unwrap()(ws_pointer.point(), bytes.to_vec()).await;
                    }
                },
                Message::Close(close_frame) => {
                    if ws.close.is_some() {
                        let callback = ws.close.as_ref().unwrap();

                        if close_frame.is_none() {
                            callback(None).await;

                            continue;
                        }

                        let close = close_frame.unwrap();

                        callback(Some(Reason{
                            code: close.code.into(),
                            message: close.reason.to_string()
                        })).await
                    }
                },
                Message::Frame(_) => {
                    // When reading frame will not be called...
                },
            }
        }

        Ok(())
    }
}

    
