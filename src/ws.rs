
use std::alloc::GlobalAlloc;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::{io::{Error, Result}, pin::Pin};

use bytes::Bytes;
use event_emitter_rs::EventEmitter;
use futures_util::future::BoxFuture;
// use event_emitter_rs::AsyncEventEmitter;
use futures_util::{SinkExt, StreamExt};
use futures_util::{stream::SplitSink};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::runtime::Runtime;
use tokio::{io::{AsyncRead, AsyncWrite, BufReader}};
use tokio_tungstenite::WebSocketStream;
use tungstenite::handshake::server::Callback;
use tungstenite::{Message, Utf8Bytes};

use async_event_emitter::AsyncEventEmitter;

use crate::request::Request;
use crate::response::{new_response, Response};
use crate::HTTP;

pub const SEC_WEB_SOCKET_ACCEPT_STATIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";


pub struct Ws {
}

impl <'a>Ws {
}