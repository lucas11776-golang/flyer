use std::{collections::HashMap, io::{Error, Result}};
use urlencoding::decode;


pub type Headers = HashMap<String, String>;
pub type Values = HashMap<String, String>;
pub type Files = HashMap<String, File>;


// #[derive(Debug)]
pub struct File {
    name: String,
    content_type: String,
    content: Vec<u8>,
}

struct MultipartForm {
    values: Values,
    files: Files,
}

// #[derive(Debug)]
pub struct Request {
    pub host: String,
    pub method: String,
    pub path: String,
    pub parameters: Values,
    pub protocol: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub values: Values,
    pub files: Files,
}

fn parse_parameters(parameters: String) -> Headers {
    let mut params: Values = Values::new();

    parameters.trim()
        .split("&")
        .for_each(|item| {
            let param: Vec<String> = item.to_string()
                .split("=")
                .map(|val| val.to_string())
                .collect();

            params.insert(
                param.get(0).get_or_insert(&"".to_owned()).to_string(),
                decode(param.get(1).get_or_insert(&"".to_owned())).unwrap().to_string()
            );
        });

    return params;
}

fn parse_urlencoded_form(body: String) -> Values {
    return parse_parameters(body);
}

struct MultipartField {
    content_disposition: String,
    name: String,
    filename: String,
    content_type: String,
    content: String,
}

fn parse_content_disposition(content_disposition: String) -> Result<MultipartField> {
    return Ok(MultipartField {
        content_disposition: "".to_owned(),
        name: "".to_owned(),
        filename: "".to_owned(),
        content_type: "".to_owned(),
        content: "".to_owned(),
    })
}

fn parse_multipart_field(field: String) -> Result<MultipartField> {
    let parts: Vec<String> = field.split("\r\n")
        .map(|x| x.to_string())
        .collect();


    if parts.len() < 2 {
        panic!("invalid multipart form field"); // TODO: remove.
    }

    let mut multipart_field = parse_multipart_field(field)?;

    return Ok(multipart_field);
}

// TODO: Parse multipart base on (https://www.rfc-editor.org/rfc/rfc7578)
fn parse_multipart_form(mut boundary: String, body: String) -> Result<MultipartForm> {
    // TODO: current parse does not validate multipart structure
    let mut values: Values = Values::new();
    let mut files: Files = Files::new();

    boundary = parse_parameters(boundary).get("boundary")
        .get_or_insert(&"".to_owned())
        .to_string();

    body.split(&format!("--{}", boundary))
        .map(|x| x.trim().trim_end_matches("--").to_string())
        .filter(|x| x != "")
        .for_each(|x| {
            let field: MultipartField = parse_multipart_field(x).unwrap();

            if field.filename.is_empty() {
                values.insert(field.name, field.content);
                return;
            }

            files.insert(field.name.clone(), File {
                name: field.name,
                content_type: field.content_type,
                content: field.content.into(),
            });
        });

    return Ok(MultipartForm {
        values: values,
        files: files,
    });
}

// TODO - parse http base on (https://www.rfc-editor.org/rfc/rfc2616)
pub fn parse(http: String) -> Result<Request> {
    let parts: Vec<String> = http.split("\r\n\r\n").map(|x| x.to_string()).collect();

    if parts.len() < 2 {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid request".to_string()));
    }

    let raw: &mut Vec<String> = &mut parts.get(0)
        .unwrap()
        .split("\r\n").map(|x| x.to_string())
        .collect();
    let top: Vec<String> = raw.get(0).unwrap()
        .split(" ")
        .map(|x| x.to_string())
        .collect();

    if top.len() != 3 {
        return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid request".to_string()));
    }

    let mut host: String = "".to_string();
    let mut headers: Headers  = Headers::new();
    let body: String = parts[1..].join("\r\n\r\n").to_string();

    for header in raw[1..].to_vec() {
        let segment: Vec<String> = header.split(":")
            .map(|x| x.to_string().trim().to_string())
            .collect();

        if top.len() < 2 {
            return Err(Error::new(std::io::ErrorKind::InvalidData, "invalid request".to_string()));
        }

        let key: String = segment.get(0).unwrap().to_owned();
        let value: String = segment[1..].join(":").to_owned();

        if key == "Host" {
            host = value;

            continue;
        }

        headers.insert(key, value);
    }

    let req: Request = Request{
        host: host,
        method: top.get(0).unwrap().to_string(),
        path: top.get(1).unwrap().to_string(),
        parameters: Values::new(),
        protocol: top.get(2).unwrap().to_string(),
        headers: headers,
        body: "".into(),
        values: HashMap::new(),
        files: HashMap::new(),
        // server: None,
    };

    return parse_request_body(req,  body);
}

fn parse_request_body(mut req: Request, body: String) -> Result<Request> {
    let content_type_value: Vec<String> =  req.header("Content-Type")
        .split(";")
        .map(|x| x.trim().to_string())
        .collect();
    let content_type = content_type_value.get(0).get_or_insert(&"".to_owned()).to_string();
    let boundary = content_type_value.get(1).get_or_insert(&"".to_owned()).to_string();

    match content_type {
        val if val == "application/x-www-form-urlencoded".to_owned() => (|| {
            req.values = parse_urlencoded_form(body);
        })(),

        val if val == "multipart/form-data".to_owned() => (|| {
            let form: MultipartForm = parse_multipart_form(boundary, body).unwrap();

            req.values = form.values;
            req.files = form.files;
        })(),

        _ =>  (|| {
            req.body = body.into();
        })()
    };

    return Ok(req);
}

impl Request {
    pub fn header(&self, key: &str) -> String {
        return self.headers.get(key).get_or_insert(&"".to_string()).to_string()
    }

    pub fn parameter(&self, key: &str) -> String {
        return self.parameters.get(key).get_or_insert(&"".to_string()).to_string()
    }

    pub fn value(&self, key: &str) -> Option<String> {
        return Some(self.values.get(key).get_or_insert(&"".to_owned()).to_string());
    }

    pub fn file(&self, key: &str) -> Option<&File> {
        return self.files.get(key);
    }
}