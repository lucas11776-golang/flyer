use bytes::Bytes;
use h2::{server, server::{SendResponse}};
use http::{HeaderMap, Request as H2Request, Response as H2Response};
use reqwest::Url;
use std::{collections::HashMap, io::Result};
use std::net::SocketAddr;
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{response::{self, new_response, Response}, HTTP as Server};
use crate::request::{Headers, Request};
use crate::utils::url::parse_query_params;

pub const H2_PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

pub struct Handler<'a> {
    server: &'a mut Server,
    addr: SocketAddr,
}

impl <'a> Handler<'a> {
    pub async fn handle<RW>(server: &'a mut Server, rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> std::io::Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + std::marker::Send
    {
        let mut handler: Handler<'_> = Handler{
            server,
            addr: addr
        };

        handler.connect(rw).await;

        return Ok(());
    }

    async fn connect<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>)
    where
        RW: AsyncRead + AsyncWrite + Unpin + std::marker::Send
    {
        let mut conn = server::handshake(&mut rw).await.unwrap();

        while let Some(result) = conn.accept().await {
            let (req, response) = result.unwrap();

            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let _ = self.listen( req, response).await;
                });
            });
        }
    }

    fn hashmap_to_headers(&mut self, map: &HeaderMap) -> Headers {
        let mut headers = Headers::new();

        for (k, v) in map.iter() {
            headers.insert(
                k.as_str().to_string(),
                v.to_str().unwrap_or_default().to_string()
            );
        }

        return headers;
    }

    fn response_to_h2_response(&mut self, response: &mut Response) -> Result<H2Response<()>> {
        let mut builder = H2Response::builder()
            .status(response.status_code)
            .header("Content-Length", format!("{}", response.body.len()));

        for (k, v) in &mut response.headers {
            builder = builder.header(k.clone(), v.clone());
        }

        return Ok(builder.body(()).unwrap());
    }

    // TODO: when make request from https and change http the request hangs mush check if it has change
    // if it has change must close the connection.
    fn write_response(&mut self, response: &mut Response, mut send: SendResponse<Bytes>) -> Result<()> {
        let res = self.response_to_h2_response(response).unwrap();
        
        send.send_response(res, false)
            .unwrap()
            .send_data(Bytes::from(response.body.clone()), true)
            .unwrap();

        return Ok(());
    }

    fn handle_request(&mut self, mut req: Request, send: SendResponse<Bytes>) -> Result<()> {
        match self.server.router.match_web_routes(&mut req) {
            Some(route) => {
                let mut res = response::new_response();

                (route.route)(&mut req, &mut res);

                self.write_response(&mut res, send)?;
            },
            None => {
                match self.server.router.not_found_callback {
                    Some(callback) => {
                        let mut res = new_response();

                        callback(&mut req, &mut res);

                        self.write_response(&mut res, send)?;
                    },
                    None => {
                        self.write_response(new_response().status_code(404), send)?;
                    },
                }
            },
        }

        return Ok(())
    }

    async fn listen(&mut self , request: H2Request<h2::RecvStream>, send: SendResponse<Bytes>) -> std::io::Result<()> {
        let method = request.method().to_string();
        let path = Url::parse(request.uri().to_string().as_str()).unwrap().path().to_string();
        let parameters = parse_query_params(request.uri().query().unwrap_or(""));
        let mut body = Vec::new();
        let headers = self.hashmap_to_headers(request.headers());
        let mut recv = request.into_body();

        while let Some(chunk) = recv.data().await.transpose().unwrap() {
            body.extend_from_slice(&chunk);
        }

        let host = headers
            .get("host")
            .cloned()
            .or_else(|| headers.get(":authority").cloned())
            .unwrap_or_default();

        println!("\r\n\r\nStream ID: {}\r\n\r\n", send.stream_id().as_u32());

        self.handle_request(Request {
            host: host,
            method: method,
            path: path,
            parameters: parameters,
            protocol: "HTTP/2.0".to_string(),
            headers: headers,
            body: body,
            values: HashMap::new(),
            files: HashMap::new(),
        }, send)?;

        Ok(())
    }
}