pub mod http1;
pub mod http2;
pub mod http3;

use std::io::Result;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::{HTTP as Server};
use crate::request::Request;
use crate::response::{Response, new_response, parse};

pub struct HTTP { }

// TODO: HTTP2 write response not the same HTTP1 must find a better way.
impl HTTP {
    pub async fn web<'a, RW>(server: &'a mut Server, buffer: &mut BufReader<RW>, req: &mut Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin
    {
        match server.router.match_web_routes(req) {
            Some(route) => {
                let res = &mut new_response();

                (route.route)(req, res);
                
                let _ = buffer.write( parse(res)?.as_bytes()).await?;
                
                Ok(())
            },
            None => {
                match server.router.not_found_callback {
                    Some(route) => {
                        let mut res: Response = new_response();

                        route(req, &mut res);

                        let _ = buffer.write(parse(&mut res)?.as_bytes()).await;

                        Ok(())
                    },
                    None => {
                        let mut res: Response = new_response();

                        res.status_code(404);

                        let _ = buffer.write(parse(&mut res)?.as_bytes()).await;

                        Ok(())
                    },
                }
            },
        }
    }

    pub async fn socket<'a, RW>(server: &'a mut HTTP, buffer: &mut BufReader<RW>, req: &mut Request) -> Result<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin
    {
        return Ok(());
    }
}