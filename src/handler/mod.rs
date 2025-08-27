pub mod http1;
pub mod http2;
pub mod http3;

use std::io::{Result as IOResult};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

use crate::utils::url;
use crate::{HTTP as Server};
use crate::request::{File, Files, MultipartForm, Request, Values};
use crate::response::{Response, new_response, parse};

pub struct HTTP { }


use std::convert::Infallible;

use bytes::Bytes;
// Import multer types.
use futures_util::stream::once;
use futures_util::stream::Stream;
use multer::Multipart;

// fn parse_parameters(parameters: String) -> Headers {
//     let mut params: Values = Values::new();

//     parameters.trim()
//         .split("&")
//         .for_each(|item| {
//             let param: Vec<String> = item.to_string()
//                 .split("=")
//                 .map(|val| val.to_string())
//                 .collect();

//             params.insert(
//                 param.get(0).get_or_insert(&"".to_owned()).to_string(),
//                 decode(param.get(1).get_or_insert(&"".to_owned())).unwrap().to_string()
//             );
//         });

//     return params;
// }

// fn parse_urlencoded_form(body: String) -> Values {
//     return parse_parameters(body);
// }


async fn parse_multipart_form<'a>(req: &'a mut Request, boundary: &'a str) -> IOResult<MultipartForm> {
    let mut form =  MultipartForm {
        values: Values::new(),
        files: Files::new(),
    };

    let stream = once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(req.body.clone())) });

    // Create a `Multipart` instance from that byte stream and the boundary.
    let mut multipart = Multipart::new(stream, boundary);

    // Iterate over the fields, use `next_field()` to get the next field.
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

// TODO: Parse multipart base on (https://www.rfc-editor.org/rfc/rfc7578)
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
        match server.router.match_web_routes(req, &mut new_response()) {
            Some(res) => {
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

    pub async fn socket<'a, RW>(server: &'a mut HTTP, buffer: &mut BufReader<RW>, req: &mut Request) -> IOResult<()>
    where
        RW: AsyncRead + AsyncWrite + Unpin
    {
        return Ok(());
    }
}