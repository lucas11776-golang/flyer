pub mod http1;
pub mod http2;
pub mod http3;

use std::io::{Result as IOResult};
use std::convert::Infallible;

use bytes::Bytes;

use futures_util::stream::once;
use multer::Multipart;

use crate::utils::url::parse_query_params;
use crate::utils::{url, Values};
use crate::view::new_view;
use crate::ws::Ws;
use crate::{HTTP};
use crate::request::{File, Files, MultipartForm, Request};
use crate::response::Response;

pub struct RequestHandler {}

impl <'a>RequestHandler {
    pub async fn web(http: &'a mut HTTP, req: &'a mut Request, res: &'a mut Response) -> IOResult<&'a mut Response> {
        let req = parse_request_body(req).await?;

        if http.configuration.get("view_path").is_some() {
            res.view = Some(new_view(http.configuration.get("view_path").unwrap().to_string()));
        }

        if http.session_manger.is_some() {
            http.session_manger.as_mut().unwrap().handle(req, res);
        }

        Ok(
            match http.router.match_web_routes(req, res) {
                Some(_) => res,
                None => {
                    match http.router.not_found_callback {
                        Some(route) => route(req, res),
                        None => res.status_code(404),
                    }
                },
            }
        )
    }

    pub fn ws(http: &'a mut HTTP, req: &'a mut Request, ws: &'a mut Ws) -> IOResult<()> {
        Ok(())
    }  
}

pub async fn parse_request_body<'a>(req: &'a mut Request) -> IOResult<&'a mut Request> {
    let binding = req.header("content-type");
    let content_type: Vec<&str> = binding.split(";").collect();

    match content_type[0] {
        "multipart/form-data" => {
            // TODO: error invalid form-data
            if content_type.len() != 2 {
                return Ok(req);
            }

            let params = url::parse_query_params(content_type[1].trim())?;
            let form = parse_multipart_form(req, &params.get("boundary").unwrap().clone()).await?;

            req.files = form.files;
            req.values = form.values;
        },
        "application/x-www-form-urlencoded" => {
            req.values = parse_query_params(&String::from_utf8_lossy(&req.body).to_string())?;
        },
        _ => {}
    }

    Ok(req)
}

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

    Ok(form)
}