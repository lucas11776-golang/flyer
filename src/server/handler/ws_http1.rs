use std::io::Result;
use std::pin::Pin;

use openssl::sha::Sha1;
use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::{request::Request, response::Response, router::{Route, WsRoute}, ws::SEC_WEB_SOCKET_ACCEPT_STATIC};

pub(crate) struct Handler<'a, RW> {
    rw: Pin<&'a mut BufReader<RW>>,
}

impl <'a, RW>Handler<'a, RW>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
{
    pub fn new(rw: Pin<&'a mut BufReader<RW>>) -> Handler<'a, RW> {
        return Self {
            rw: rw
        }
    }

    pub async fn handle(self, route: &'a mut Route<Box<WsRoute>>, request: Request) -> Result<()> {
        println!("Handle HTTP1.1 Protocol Websocket");

        Ok(())
    }

    fn websocket_handle_shake_headers(&mut self, sec_websocket_key: String, res: &'a mut Response) -> &'a mut Response {
        // TODO: handshake...
        return res.status_code(101)
            .header("Upgrade".to_owned(), "websocket".to_owned())
            .header("Connection".to_owned(), "Upgrade".to_owned())
            .header("Sec-WebSocket-Accept".to_owned(), self.get_sec_web_socket_accept(sec_websocket_key));
    }

    fn get_sec_web_socket_accept(&mut self, key: String) -> String {
        let mut hasher = Sha1::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        
        // TODO: use the new implementation...
        return base64::encode(&hasher.finish())
    }
}


