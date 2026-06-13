use anyhow::Result;
use bytes::Bytes;
use multer::{Field, Multipart};
use tokio_util::io::ReaderStream;
use serde_json::{Value, from_str};
use std::collections::HashMap;

pub type JsonValues = HashMap<String, Value>;
pub type Names = Vec<String>;

use crate::{
    request::{Request, form::File},
    utils::{Values, url::parse_query_params}
};

// TODO: refactor and add to folders.
pub(crate) async fn parse_content_type(req: &mut Request) -> Result<()> {
    match req.content_type().to_lowercase().as_str() {
        "application/x-www-form-urlencoded" => parse_form_urlencoded( req).await.unwrap(),
        "multipart/form-data" => parse_multipart_form(req).await.unwrap(),
        "application/json" => parse_json_form(req).unwrap(),
        _ => {}
    }

    if let Some(method) = req.form.values.get("_method") {
        req.method = method.to_uppercase();
    }
    
    if let Some(method) = req.form.values.get("__METHOD__") {
        req.method = method.to_uppercase();
    }

    return Ok(());
}

pub(crate) fn get_multipart_header_boundary(header: String) -> std::io::Result<String> {
    let content_type: Vec<&str> = header.split(";").collect();
    let content_type_piece = content_type.get(1).unwrap().to_string();
    let boundary =   parse_query_params(content_type_piece.trim())
        .get("boundary")
        .unwrap()
        .to_string();
    
    return Ok(boundary);
}

async fn parse_multipart_form(req: &mut Request) -> Result<()> {
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

        req.form.files.insert(name, File::create(filename.as_str(), content_type.as_str(), data));
    }

    return Ok(());
}

async fn parse_form_urlencoded(req: &mut Request) -> std::io::Result<()> {
    let values = parse_query_params(String::from_utf8(req.body.clone()).unwrap().as_str());

    for (k, v) in values {
        req.form.values.insert(k, v);
    }

    return Ok(());
}

// TODO: Need refactor to simplify
pub fn parse_json_form(req: &mut Request) -> Result<()> {
    let json: JsonValues = from_str(&String::from_utf8_lossy(&req.body).to_string()).unwrap();

    for (k, v) in json {
        let mut names: Names = Names::default();

        names.push(String::from(k));

        for (k_insert, v_insert) in parse_json_form_value(names, &v) {
            req.form.values.insert(String::from(k_insert), v_insert);
        }
    }

    return Ok(())
} 

pub fn parse_json_form_name(names: &Names) -> String {
    let mut parsed: Names = Default::default();

    for (i, v) in names.iter().enumerate() {
        if i == 0 {
            parsed.push(v.to_string());
        } else {
            parsed.push(format!("[{}]", v));
        }
    }

    return parsed.join("");
}

pub fn parse_json_form_value(names: Names, value: &Value) -> Values {
    let mut form: Values = Values::default(); 

    match value {
        Value::Array(values) => {
            for (k_array, v_array) in values.iter().enumerate() {
                let mut names_array = names.clone();
                
                names_array.push(k_array.to_string());

                for (k_insert, v_insert) in parse_json_form_value(names_array, v_array) {
                    form.insert(String::from(k_insert), v_insert);
                }
            }
        },
        Value::Object(map) => {
            for (k_map, v_map) in map {
                let mut names_map = names.clone();

                names_map.push(String::from(k_map));

                for (k_insert, v_insert) in parse_json_form_value(names_map, v_map) {
                    form.insert(String::from(k_insert), v_insert);
                }
            }
        },
        _ => {
            form.insert(parse_json_form_name(&names), String::from(value.to_string().trim_matches('"')));
        }
    }

    return form;
}
