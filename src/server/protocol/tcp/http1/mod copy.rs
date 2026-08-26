use std::io::{Error, ErrorKind, Write};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use anyhow::Result;
use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

use crate::request::form::Form;
use crate::request::Request;
use crate::response::{self, LOCAL_RESPONSE, LoggerTWrapper, Response, Writer, WriterT};
use crate::server::protocol::TcpHandler;
use crate::server::Server;
use crate::server::protocol::tcp::http1::ws::Ws;
use crate::utils::future::SendFuture;
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
}


impl <RW>Http1Writer<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
{
    pub fn new(rw: Instance<BufReader<RW>>, res: Instance<Response>) -> Self {
        Self {
            rw: rw,
            res: res
        }
    }
}

impl <RW>WriterT for Http1Writer<RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
{
    async fn write(&self, data: Bytes) -> Result<()> {
        let rw = self.rw.as_mut();
        let res = self.res.as_mut();

        if !res.has_sent {
            res.has_sent = true;

            Http1::write_header(rw, res)
                .await
                .unwrap();
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
            return Ws::new(self.server.clone(), self.addr).handle(rw, req).await;
        }

        let mut res = Response::new();

        let writer = Http1Writer::new(
            Instance(&mut rw as *mut BufReader<RW>),
            Instance(&mut res as *mut Response)
        );

        res.writer = Some(Arc::new(LoggerTWrapper::new(writer)));

        let (_req, mut res) = self
            .server
            .as_mut()
            .on_http(req, res)
            .await;

        println!("HAS SENT -> {}", res.has_sent);

        if res.has_sent {
            return Ok(());
        }

        let content_length = res.content.len();
        let mut res = res.set_header("Content-Length", content_length.to_string());

        Self::write_response(&mut rw, &mut res).await







        // // TODO: find out why LOCAL_RESPONSE != res (maybe cloning).
        // let mut res = Response::new(
        //     LoggerTWrapper::new(
        //             Http1Writer::new(Instance(&mut rw as *mut BufReader<RW>))
        //         )
        // );

        // let (_req, res) = LOCAL_RESPONSE
        //     .scope(Instance(&mut res as *mut Response), async {
        //         let (req, mut res) = self
        //             .server
        //             .as_mut()
        //             .on_http(req, res)
        //             .await;

        //         res.has_sent = LOCAL_RESPONSE.get().as_ref().has_sent;




        //         println!("TESTING -... {:?}", LOCAL_RESPONSE.get().as_ref().view.is_some());

        //         (req, res)    
        //     })
        //     .await
        //     ;


        // println!("RESPONSE HAS SENT {} {:?}", res.has_sent, "");


        // if res.has_sent {
        //     return Ok(())
        // }

        // let content_length = res.content.len();
        // let mut res = res.set_header("Content-Length", content_length.to_string());

        // Self::write_response(&mut rw, &mut res).await





        // todo!()




        // let (tx, mut rx) = unbounded_channel::<Bytes>();

        // let mut writer = ResponseWriter::new(Http1Writer::new(tx));
        // let writer_instance = Instance(&mut writer as *mut ResponseWriter);

        // let mut res = Response::new(writer_instance);
        // let rw_instance = Instance(&mut rw as *mut BufReader<RW>);
        // let res_instance = Instance(&mut res as *mut Response);

        // let sent = Arc::new(AtomicBool::new(false));
        // let sent_task = Arc::clone(&sent);

        // tokio::spawn(async move {
        //     let rw = rw_instance.as_mut();
        //     let res = res_instance.as_mut();

        //     while let Some(msg) = rx.recv().await {

        //         // println!("Testing ------ 1");

        //         // if !sent_task.load(Ordering::Relaxed) {
        //         //     sent_task.store(true, Ordering::Relaxed);



        //         //     let a= Self::write_header(rw, res)
        //         //         .await
        //         //         ;

        //         //     if let Err(err) = a {
        //         //         println!("ERROR {:?}", err);
        //         //     }


        //         //     println!("Testing ------ Wait");
        //         // }

        //         println!("Testing ------ 2");

        //         rw
        //             .write_all(&msg)
        //             .await
        //             .unwrap();
        //     }
        // });


        // let (_, res) = self
        //     .server
        //     .as_mut()
        //     .on_http(req, res)
        //     .await;

        // println!("!!!! DONE EXECUTING !!!!");


        // if sent.load(Ordering::Relaxed) {
        //     return Ok(())
        // }

        // let content_length = res.content.len();
        // let mut res = res.set_header("Content-Length", content_length.to_string());

        // Self::write_response(&mut rw, &mut res).await
    }
}

impl Http1 {
    async fn deserialize<RW>(&mut self, rw: &mut BufReader<RW>) -> Result<Request>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync,
    {
        let mut buffer = BytesMut::with_capacity(MAX_HEADER_LENGTH);

        // Read stream until complete HTTP headers are loaded
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

        // Split off headers from leftover body data
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
            let name_lower = h.name.to_ascii_lowercase();

            if name_lower == "content-length" {
                content_length = val_str.parse().unwrap_or(0);
            } else if name_lower == "transfer-encoding" && val_str.contains("chunked") {
                is_chunked = true;
            }

            headers.insert(name_lower, val_str.to_string());
        }

        let body = if is_chunked {
            self.read_chunked_body(rw, leftover_body).await?
        } else {
            self.read_fixed_body(rw, leftover_body, content_length)
                .await?
        };

        let raw_url = parsed_req.path.unwrap_or("");
        let (path, queries) = match raw_url.find('?') {
            Some(i) => (&raw_url[..i], parse_query(&raw_url[i + 1..])),
            None => (raw_url, Values::new()),
        };

        let host = headers.get("host").cloned().unwrap_or_default();

        Ok(Request {
            server: self.server.clone(),
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
        &mut self,
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

        let cl_usize = content_length as usize;

        // If leftover buffer already contains the entire body (or more)
        if leftover.len() >= cl_usize {
            let body_bytes = leftover.split_to(cl_usize);
            return Ok(body_bytes.to_vec());
        }

        // Pre-allocate required capacity once up front
        let mut body = Vec::with_capacity(cl_usize);
        let leftover_len = leftover.len();
        body.extend_from_slice(&leftover);

        let remaining = cl_usize - leftover_len;
        let mut limited = rw.take(remaining as u64);
        limited.read_to_end(&mut body).await?;

        Ok(body)
    }

    async fn read_chunked_body<RW>(
        &mut self,
        rw: &mut BufReader<RW>,
        mut buf: BytesMut,
    ) -> Result<Vec<u8>>
    where
        RW: AsyncRead + Unpin + Send + Sync,
    {
        let mut body = Vec::new();

        loop {
            // Find CRLF line boundary for chunk size
            let line_end = loop {
                if let Some(pos) = buf.windows(2).position(|w| w == b"\r\n") {
                    break pos;
                }
                if rw.read_buf(&mut buf).await? == 0 {
                    return Err(
                        Error::new(ErrorKind::UnexpectedEof, "Truncated chunk size").into(),
                    );
                }
            };

            let size_bytes = buf.split_to(line_end);
            buf.advance(2); // Skip \r\n

            let size_str = std::str::from_utf8(&size_bytes)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 in chunk size"))?;
            let hex_str = size_str.split(';').next().unwrap_or("").trim();
            let chunk_size = usize::from_str_radix(hex_str, 16)
                .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid hex chunk size"))?;

            if chunk_size == 0 {
                // Read trailing CRLF for end chunk if needed
                while buf.len() < 2 {
                    if rw.read_buf(&mut buf).await? == 0 {
                        break;
                    }
                }
                break;
            }

            // Ensure full chunk payload + trailing CRLF (2 bytes) is buffered
            let total_needed = chunk_size + 2;
            while buf.len() < total_needed {
                if rw.read_buf(&mut buf).await? == 0 {
                    return Err(
                        Error::new(ErrorKind::UnexpectedEof, "Truncated chunk body").into(),
                    );
                }
            }

            body.extend_from_slice(&buf[..chunk_size]);
            buf.advance(total_needed); // Advance past chunk data + CRLF
        }

        Ok(body)
    }

    fn serialize(res: &Response) -> Vec<u8> {
        let content_length = res.content.len();
        let mut serialized = Vec::with_capacity(128 + (res.headers.len() * 32) + content_length);
        let status_text = http::StatusCode::from_u16(res.status_code)
            .map(|s| s.canonical_reason().unwrap_or("OK"))
            .unwrap_or("OK");
        let _ = write!(serialized, "HTTP/1.1 {} {}\r\n", res.status_code, status_text);

        for (k, v) in &res.headers {
            if !k.eq_ignore_ascii_case("content-length") {
                let _ = write!(serialized, "{}: {}\r\n", k, v);
            }
        }

        let _ = write!(serialized, "Content-Length: {}\r\n\r\n", content_length);

        serialized.extend_from_slice(&res.content);

        serialized
    }

    async fn write_header<RW>(rw: &mut BufReader<RW>, res: &mut Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let content_length = res.content.len();
        let mut serialized = Vec::with_capacity(128 + (res.headers.len() * 32) + content_length);
        let status_text = http::StatusCode::from_u16(res.status_code)
            .map(|s| s.canonical_reason().unwrap_or("OK"))
            .unwrap_or("OK");
        let _ = write!(serialized, "HTTP/1.1 {} {}\r\n", res.status_code, status_text);

        for (k, v) in &res.headers {
            let _ = write!(serialized, "{}: {}\r\n", k, v);
        }

        let _ = write!(serialized, "\r\n");


        rw
            .write_all(&serialized)
            .await
            // .map_err(Into::into)
            .unwrap();


        println!("TESTING write_header");


        let a = rw
            .flush()
            .await
            // .map_err(Into::into)
            .unwrap()
            ;


        // println!("DATA SENT");

        // a

        Ok(())
    }

    async fn write_response<RW>(rw: &mut BufReader<RW>, res: &mut Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        Self::write_header(rw, res)
            .await
            .unwrap();

        rw
            .flush()
            .await
            .map_err(Into::into)
    }
}