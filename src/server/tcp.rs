use std::{
    io::Result,
    net::SocketAddr,
};
use std::pin::{pin, Pin};

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader};

use crate::request::Request;
use crate::response::Response;
use crate::server::handler::{http1, http2, ws_http1};
use crate::server::handler::http2::H2_PREFACE;
use crate::server::{HTTP1, HTTP2, Protocol};
use crate::HTTP;

pub(crate) struct TcpServer<'a> {
    listener: TcpListener,
    acceptor: Option<TlsAcceptor>,
    http: &'a mut HTTP
}

impl <'a>TcpServer<'a> {
    pub async fn new(http: &'a mut HTTP, tls: Option<TlsAcceptor>) -> Result<TcpServer<'a>> {
        return Ok(TcpServer{
            listener: TcpListener::bind(format!("{}", http.address())).await.unwrap(),
            acceptor: tls,
            http: http,
        });
    }

    pub async fn listen(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tokio_scoped::scope(|scope| {
                        scope.spawn(self.new_connection(stream, addr));
                    });
                },
                Err(_) => {}, // TODO: Log
            }
        }
    }

    async fn new_connection<RW>(&mut self, stream: RW, addr: SocketAddr)
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match &self.acceptor {
            Some(acceptor) => {
                let _ = match acceptor.accept(stream).await {
                    Ok(stream) => self.handle_connection(pin!(BufReader::new(stream)), addr).await.unwrap(),
                    Err(_) => {}, // TODO: Log
                };
            },
            None => self.handle_connection(pin!(BufReader::new(stream)), addr).await.unwrap(),
        }
    }

    async fn handle_connection<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>, addr:  SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        match self.connection_protocol(rw.fill_buf().await?) {
            HTTP2 => self.http_2_protocol(rw, addr).await.unwrap(),
            _ => self.http_1_protocol(rw, addr).await.unwrap()
        }

        Ok(())
    }

    fn connection_protocol(&'_ mut self, buffer: &[u8]) -> Protocol {
        match buffer.len() >= H2_PREFACE.len() && &buffer[..H2_PREFACE.len()] == H2_PREFACE {
            true => HTTP2,
            false => HTTP1,
        }
    }

    async fn handle_web_socket<RW>(&mut self, rw: Pin<&mut BufReader<RW>>, req: &mut Request, res: &mut Response) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let (mut handler, req, res) = ws_http1::Handler::new(rw, req, res).await.unwrap();
        let result = self.http.router.match_ws_routes(req, res).await;

        if result.is_none() {
            return Ok(())
        }

        let (route, req, res) = result.unwrap();

        return Ok(handler.handle(route, req, res).await.unwrap());
    }
 
    async fn http_1_protocol<RW>(&mut self, mut rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = http1::Handler::new(Pin::new(&mut rw), addr);
        let result = handler.handle().await;

        if result.is_none() {
            return Ok(());
        }

        let mut req = result.unwrap().unwrap();
        let mut res = Response::new();

        if req.header("upgrade") == "websocket" {
            (req, res) = self.handle_session(req, res).unwrap();

            self.handle_web_socket(rw, &mut req, &mut res).await.unwrap();

            return Ok(())
        }

        (req, res) = self.handle(req, res).await.unwrap();

        return Ok(handler.write(&mut res).await.unwrap());
    }

    async fn http_2_protocol<RW>(&mut self, rw: Pin<&mut BufReader<RW>>, addr: SocketAddr) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static
    {
        let mut handler = http2::Handler::new(addr, rw).await;

        while let Some(result) = handler.handle().await {
            tokio_scoped::scope(|scope| {
                scope.spawn(async {
                    let (request, send) = result.unwrap();
                    let mut req = handler.get_http_request(request).await.unwrap();
                    let mut res = Response::new();

                    (req, res) = self.handle(req, res).await.unwrap();

                    handler.write(send, &mut res).await.unwrap();
                });
            });
        }

        return Ok(())   
    }

    async fn handle<'h>(&mut self, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        (req, res) = self.handle_session(req, res).unwrap();
        let resp = self.http.router.match_web_routes(&mut req, &mut res).await;

        if resp.is_none() && self.http.assets.is_some() {
            (req, res) = self.http.assets.as_mut().unwrap().handle(req, res).unwrap();
        }

        self.http.render_response_view(&mut res);

        return Ok(self.handle_session_cleanup(req, res).await.unwrap());
    }

    fn handle_session(&mut self, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        if !req.is_asset() {
            self.http.session_manager
                .as_mut()
                .unwrap()
                .handle(&mut req, &mut res)
                .unwrap();
        }

        return Ok((req, res));
    }

    async fn handle_session_cleanup(&mut self, mut req: Request, mut res: Response) -> Result<(Request, Response)> {
        if self.http.session_manager.is_some() {
            if !req.is_asset() {
                self.http.session_manager
                    .as_mut()
                    .unwrap()
                    .cleanup(&mut req, &mut res)
                    .unwrap();
            }
        }

        return Ok((req, res));
    }

}