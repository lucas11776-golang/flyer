
use std::io::Result;

use bytes::Bytes;
use multer::{Field, Multipart};
use tokio_util::io::ReaderStream;

use crate::{
    request::{Request, form::File},
    utils::url::parse_query_params
};

pub(crate) async fn parse_content_type(req: Request) -> std::io::Result<Request> {
    if req.method == "POST" || req.method == "PATCH" || req.method == "PUT" {
        return Ok(
            match req.content_type().as_str() {
                "application/x-www-form-urlencoded" => parse_form_urlencoded(req).await.unwrap(),
                "multipart/form-data" => parse_multipart_form(req).await.unwrap(),
                _ => req
            }
        );
    }

    return Ok(req);
}

fn get_multipart_header_boundary(header: String) -> std::io::Result<String> {
    let content_type: Vec<&str> = header.split(";").collect();
    let content_type_piece = content_type.get(1).unwrap().to_string();
    let boundary =   parse_query_params(content_type_piece.trim()).unwrap()
        .get("boundary")
        .unwrap()
        .to_string();
    return Ok(boundary);
}

async fn parse_multipart_form(mut req: Request) -> std::io::Result<Request> {
    let boundary = get_multipart_header_boundary(req.header("content-type")).unwrap();
    let body = req.body.clone();
    let stream = ReaderStream::new(body.as_slice());
    let mut multipart = Multipart::new(stream,  boundary);

    while let Some(field) = multipart.next_field().await.or::<Result<Option<Field>>>(Ok(None)).unwrap() {
        if field.file_name().is_none() {
            req.form.values.insert(
                field.name().as_mut().unwrap().to_string(),
                field.text().await.or::<Result<String>>(Ok("".to_string())).unwrap().to_string(),
            );

            continue;
        }

        let name = field.name().as_mut().unwrap().to_string();
        let filename = field.file_name().as_mut().unwrap().to_string();
        let content_type = field.content_type().as_mut().unwrap().to_string();
        let data = field.bytes().await.as_mut().or::<&mut Bytes>(Ok(&mut bytes::Bytes::new())).unwrap().to_vec();
        
        if data.len() == 0 {
            continue;
        }

        req.form.files.insert(name, File::new(filename.as_str(), content_type.as_str(), data));
    }

    return Ok(req);
}

async fn parse_form_urlencoded(mut req: Request) -> std::io::Result<Request> {
    let values = parse_query_params(String::from_utf8(req.body.clone()).unwrap().as_str()).unwrap();

    for (k, v) in values {
        req.form.values.insert(k, v);
    }

    return Ok(req);
}