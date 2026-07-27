use std::net::SocketAddr;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose;
use futures::StreamExt;
use openssl::sha::Sha1;
use tokio::io::{AsyncWriteExt, BufReader};
use  tokio::io::{AsyncRead, AsyncWrite};

use crate::request::Request;
use crate::response::Response;
use crate::server::protocol::tcp::http1::Http1;
use crate::websocket::SEC_WEB_SOCKET_ACCEPT_STATIC;
use crate::{server::{Server}, utils::mem::Instance};

pub struct Ws {
    server: Instance<Server>,
    addr: SocketAddr,
}


use tungstenite::Utf8Bytes;
use tungstenite::{Message, protocol::Role::Server as RoleServer};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::unbounded_channel;



use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::WebSocketStream;

pub(crate) enum Type {
    Close,
    Binary,
    Text,
    Ping,
    Pong,
}

pub(crate) struct Payload {
    pub method: Type,
    pub data: Vec<u8>
}

impl Ws {
    pub fn new(server: Instance<Server>, addr: SocketAddr) -> Self {
        Self {
            server: server,
            addr: addr,
        }
    }

    pub async fn handle<RW>(&mut self, mut rw: BufReader<RW>, mut req: Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        if let Err(err) = Self::handshake(&mut rw, &mut req).await {
            return Err(err.into());
        }

        let (sink, stream) = WebSocketStream::from_raw_socket(rw, RoleServer, None)
            .await
            .split();

        let (tx, rx) = unbounded_channel::<Payload>();

        // res.ws = Some(Box::new(Writer{sender: tx}));


        todo!()
    }
}

impl Ws {
    async fn handshake<RW>(rw: &mut BufReader<RW>, req: &mut Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin + Send + Sync
    {
        let res = Response::new()
            .status_code(101)
            .set_header("Upgrade", "websocket")
            .set_header("Connection", "Upgrade")
            .set_header("Sec-WebSocket-Accept", Self::get_sec_web_socket_accept(req.header("sec-websocket-key")).as_str());

        rw
            .write_all(Http1::serialize(&res).as_slice())
            .await
            .unwrap();

        return rw
            .flush()
            .await
            .map_err(|err| err.into());
    }

    fn get_sec_web_socket_accept(key: String) -> String {
        let mut hasher = Sha1::new();
        let mut accept = String::new();
        
        hasher.update(format!("{}{}", key, SEC_WEB_SOCKET_ACCEPT_STATIC).as_bytes());
        general_purpose::STANDARD.encode_string(hasher.finish(), &mut accept);
        
        return accept;
    }
}