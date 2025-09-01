pub mod http1;
pub mod http2;
pub mod http3;

use std::io::{Result as IOResult};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use bytes::Bytes;
use futures_util::stream::once;
use multer::Multipart;

use crate::utils::url;
use crate::view::new_view;
use crate::{Values, HTTP as Server};
use crate::request::{File, Files, MultipartForm, Request};
use crate::response::{new_response, parse};

pub struct HTTP { }

use std::convert::Infallible;

async fn parse_multipart_form<'a>(req: &'a mut Request, boundary: &'a str) -> IOResult<MultipartForm> {
    let mut form =  MultipartForm {
        values: Values::new(),
        files: Files::new(),
    };

    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(req.body.clone())) });

    let mut multipart = Multipart::new(stream, boundary);

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        match field.file_name().clone() {
            Some(filename) => {
                let mut file = File {
                    name: filename.to_owned(),
                    content_type: field.content_type().unwrap().to_string(),
                    size: 0,
                    content: vec![],
                };

                while let Some(chunk) = field.chunk().await.unwrap() {
                    file.content.append(&mut chunk.to_vec());
                }

                file.size = file.content.len();

                form.files.insert(field.name().unwrap().to_string(), file);
            },
            None => {
                form.values.insert(
                    field.name().unwrap().to_owned(),
                    str::from_utf8(&field.bytes().await.unwrap()).unwrap().to_string()
                );
            },
        }
    }

    return Ok(form)

}

pub async fn parse_request_body<'a>(req: &'a mut Request) -> IOResult<()> {
    let binding = req.header("Content-Type");
    let content_type: Vec<&str> = binding.split(";").collect();

    if content_type.len() != 2 {
        return Ok(());
    }

    match content_type[0] {
        "multipart/form-data" => {
            if content_type.len() != 2 {
                return Ok(());
            }

            let params = url::parse_query_params(content_type[1].trim());
            let form = parse_multipart_form(req, &params.get("boundary").unwrap().clone()).await?;

            req.files = form.files;
            req.values = form.values;
        },
        // TODO: implement url-encode
        _ => {

        }
    }

    return Ok(());
}


// TODO: HTTP2 write response not the same HTTP1 must find a better way.
impl HTTP {
    pub async fn web<'a, RW>(server: &'a mut Server, buffer: &mut BufReader<RW>, req: &mut Request) -> IOResult<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin
    {
        let mut res = new_response();

        if server.configuration.get("view_path").is_some() {
            res.view = Some(new_view(server.configuration.get("view_path").unwrap().to_string()));
        }

        match server.router.match_web_routes(req, &mut res) {
            Some(res) => {
                match &server.session_manger {
                    Some(manager) => {
                        manager.handle(req, res);
                    },
                    None => {},
                };

                let _ = buffer.write( parse(res)?.as_bytes()).await?;
                
                Ok(())
            },
            None => {
                match server.router.not_found_callback {
                    Some(route) => {
                        route(req, &mut res);

                        let _ = buffer.write(parse(&mut res)?.as_bytes()).await;

                        Ok(())
                    },
                    None => {
                        res.status_code(404);

                        let _ = buffer.write(parse(&mut res)?.as_bytes()).await;

                        Ok(())
                    },
                }
            },
        }
    }

    // TODO: implement socket protocol.
    pub async fn socket<'a, RW>(server: &'a mut HTTP, buffer: &mut BufReader<RW>, req: &mut Request) -> IOResult<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin
    {
        return Ok(());
    }
}