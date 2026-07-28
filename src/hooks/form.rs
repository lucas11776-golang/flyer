
use std::io::Cursor;

use anyhow::{Context, Result};
use bytes::Bytes;
use multer::Multipart;
use tokio_util::io::ReaderStream;
use serde_json::Value;
use std::collections::HashMap;

use crate::{
    hooks::Hook,
    request::{Request, form::File},
    response::Response,
    routing::next::Next,
    utils::url::parse_query
};

pub type JsonValues = HashMap<String, Value>;
pub type Names = Vec<String>;

pub struct MultipartFormHook;

impl MultipartFormHook {
    pub fn new() -> Self {
        return Self;
    }
}

impl Hook for MultipartFormHook {
    async fn before(&self, mut req: Request, res: Response, next: Next) -> Response {
        if let Err(_) = self.parse(&mut req).await {
            // TODO: request bad request maybe Hook should have need still thinking about it.
        }

        return next.handle(req, res);
    }

    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        return next.handle(req, res);
    }
}

impl MultipartFormHook {
    async fn parse(&self, req: &mut Request) -> Result<()> {
        let content_type = req
            .header("content-type")
            .to_lowercase();

        if content_type.starts_with("application/x-www-form-urlencoded") {
            self.parse_form_urlencoded(req).await?;
        } else if content_type.starts_with("multipart/form-data") {
            self.parse_multipart_form(req).await?;
        } else if content_type.starts_with("application/json") {
            self.parse_json_form(req).await?;
        }

        let field_method = req
            .form
            .values
            .get("_method")
            .or_else(|| req.form.values.get("__METHOD__"));

        if let Some(method) = field_method {
            req.method = method.to_uppercase();
        }

        return Ok(());
    }


    fn get_multipart_header_boundary(&self, header: &str) -> Result<String> {
        for part in header.split(';') {
            let trimmed = part.trim();
            if trimmed.to_lowercase().starts_with("boundary=") {
                let boundary = trimmed["boundary=".len()..].trim_matches('"');
                if !boundary.is_empty() {
                    return Ok(boundary.to_string());
                }
            }
        }
        anyhow::bail!("Multipart boundary parameter missing from Content-Type header")
    }

    async fn parse_multipart_form(&self, req: &mut Request) -> Result<()> {
        let boundary = self.get_multipart_header_boundary(&req.header("content-type"))?;
        
        let mut values = HashMap::new();
        let mut files = HashMap::new();

        {
            let cursor = Cursor::new(&req.body[..]);
            let stream = ReaderStream::new(cursor);
            let mut multipart = Multipart::new(stream, boundary);

            while let Some(field) = multipart.next_field().await? {
                let name = field
                    .name()
                    .unwrap_or_default()
                    .to_string();

                if let Some(filename) = field.file_name() {
                    let filename = filename.to_string();
                    let content_type = field
                        .content_type()
                        .map(|mime| mime.as_ref())
                        .unwrap_or("application/octet-stream")
                        .to_string();
                    let data: Bytes = field
                        .bytes()
                        .await?
                        .into();

                    if !data.is_empty() {
                        files.insert(name, File::create(&filename, &content_type, data));
                    }
                } else {
                    let text = field
                        .text()
                        .await
                        .unwrap_or_default();

                    values.insert(name, text);
                }
            }
        }

        req.form.values.extend(values);
        req.form.files.extend(files);
        req.body.clear();

        Ok(())
    }

    async fn parse_form_urlencoded(&self, req: &mut Request) -> Result<()> {
        let body_str = std::str::from_utf8(&req.body)
            .context("Failed to parse URL-encoded body as valid UTF-8")?;
        
        let values = parse_query(body_str);
        req.form.values.extend(values);
        req.body.clear();

        Ok(())
    }

    async fn parse_json_form(&self, req: &mut Request) -> Result<()> {
        let parsed: Value = serde_json::from_slice(&req.body)?;
        
        let mut out_map = HashMap::new();
        let mut current_path = String::new();

        match parsed {
            Value::Array(values) => {
                for (i, item) in values.into_iter().enumerate() {
                    current_path.push_str(&i.to_string());
                    self.json_to_map(item, &mut current_path, &mut out_map);
                    current_path.clear();
                }
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    current_path.push_str(&key);
                    self.json_to_map(val, &mut current_path, &mut out_map);
                    current_path.clear();
                }
            }
            _ => {}
        }

        req
            .form
            .values
            .extend(out_map);

        Ok(())
    }

    fn json_to_map(&self, value: Value, current_path: &mut String, out_map: &mut HashMap<String, String>) {
        match value {
            Value::Null => {
                out_map.insert(current_path.clone(), "null".into());
            }
            Value::Bool(b) => {
                out_map.insert(current_path.clone(), b.to_string());
            }
            Value::Number(num) => {
                out_map.insert(current_path.clone(), num.to_string());
            }
            Value::String(s) => {
                out_map.insert(current_path.clone(), s);
            }
            Value::Array(arr) => {
                let len = current_path.len();

                for (i, item) in arr.into_iter().enumerate() {
                    if len == 0 {
                        current_path.push_str(&i.to_string());
                    } else {
                        current_path.push('[');
                        current_path.push_str(&i.to_string());
                        current_path.push(']');
                    }

                    self.json_to_map(item, current_path, out_map);

                    current_path.truncate(len);
                }
            }
            Value::Object(obj) => {
                let len = current_path.len();

                for (key, val) in obj {
                    if len == 0 {
                        current_path.push_str(&key);
                    } else {
                        current_path.push('[');
                        current_path.push_str(&key);
                        current_path.push(']');
                    }

                    self.json_to_map(val, current_path, out_map);

                    current_path.truncate(len);
                }
            }
        }
    }
}