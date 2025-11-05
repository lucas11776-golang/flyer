use std::io::Result;
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite, BufReader};

use crate::request::Request;

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

    pub async fn handle(self, request: Request) -> Result<()> {
        Ok(())
    }
}


