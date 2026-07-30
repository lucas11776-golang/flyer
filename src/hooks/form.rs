use std::collections::HashMap;
use std::fmt::Write;

use anyhow::{Context, Result};
use bytes::Bytes;
use futures_util::stream;
use multer::Multipart;
use serde_json::Value;

use crate::{
    hooks::Hook,
    request::{form::File, Request},
    response::Response,
    routing::next::Next,
    utils::url::parse_query,
};

pub type JsonMap = HashMap<String, Value>;

#[derive(Default)]
pub(crate) struct FormHook;

impl FormHook {
    pub fn new() -> Self {
        Self
    }

    fn extract_boundary<'a>(&self, header: &'a str) -> Result<&'a str> {
        header
            .split(';')
            .map(str::trim)
            .find_map(|part| {
                if part.len() >= 9 && part[..9].eq_ignore_ascii_case("boundary=") {
                    let boundary = part[9..].trim_matches('"');
                    if !boundary.is_empty() {
                        return Some(boundary);
                    }
                }
                None
            })
            .context("Multipart boundary parameter missing from Content-Type header")
    }

    async fn parse(&self, req: &mut Request) -> Result<()> {
        let content_type = req.header("content-type");
        let mime = content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim();

        if mime.eq_ignore_ascii_case("application/x-www-form-urlencoded") {
            self.parse_form_urlencoded(req).await?;
        } else if mime.eq_ignore_ascii_case("multipart/form-data") {
            self.parse_multipart_form(req).await?;
        } else if mime.eq_ignore_ascii_case("application/json") {
            self.parse_json_form(req).await?;
        }

        if let Some(method) = req
            .form
            .values
            .get("_method")
            .or_else(|| req.form.values.get("__METHOD__"))
        {
            req.method = method.to_uppercase();
        }

        Ok(())
    }

    async fn parse_multipart_form(&self, req: &mut Request) -> Result<()> {
        let header = req.header("content-type");
        let boundary = self.extract_boundary(&header)?;
        let body_bytes = std::mem::take(&mut req.body);

        let stream = stream::once(async move {
            Ok::<_, std::convert::Infallible>(Bytes::from(body_bytes))
        });

        let mut multipart = Multipart::new(stream, boundary);
        let mut values = HashMap::new();
        let mut raw_files: Vec<(String, File)> = Vec::new();
        let mut field_file_counts: HashMap<String, usize> = HashMap::new();

        while let Some(field) = multipart.next_field().await? {
            let name = field.name().unwrap_or_default().to_string();

            if let Some(filename) = field.file_name() {
                let filename = filename.to_string();
                let content_type = field
                    .content_type()
                    .map(|mime| mime.as_ref())
                    .unwrap_or("application/octet-stream")
                    .to_string();

                let data: Bytes = field.bytes().await?;
                if data.is_empty() {
                    continue;
                }

                *field_file_counts.entry(name.clone()).or_default() += 1;
                raw_files.push((name, File::create(&filename, &content_type, data)));
            } else {
                let text = field.text().await.unwrap_or_default();
                values.insert(name, text);
            }
        }

        let mut files = HashMap::with_capacity(raw_files.len());
        let mut field_file_indices: HashMap<String, usize> = HashMap::new();

        for (name, file) in raw_files {
            let total_count = field_file_counts.get(&name).copied().unwrap_or(0);
            if total_count > 1 {
                let idx = field_file_indices.entry(name.clone()).or_default();
                files.insert(format!("{}[{}]", name, idx), file);
                *idx += 1;
            } else {
                files.insert(name, file);
            }
        }

        req.form.values.extend(values);
        req.form.files.extend(files);
        req.body.clear();

        Ok(())
    }

    async fn parse_form_urlencoded(&self, req: &mut Request) -> Result<()> {
        let body_bytes = std::mem::take(&mut req.body);
        let body_str = std::str::from_utf8(&body_bytes)
            .context("Failed to parse URL-encoded body as valid UTF-8")?;

        let values = parse_query(body_str);
        req.form.values.extend(values);
        req.body.clear();

        Ok(())
    }

    async fn parse_json_form(&self, req: &mut Request) -> Result<()> {
        let parsed: Value = serde_json::from_slice(&req.body)?;
        let mut out_map = HashMap::new();
        let mut current_path = String::with_capacity(32);

        match parsed {
            Value::Array(values) => {
                for (i, item) in values.into_iter().enumerate() {
                    current_path.clear();
                    let _ = write!(current_path, "{}", i);
                    self.json_to_map(item, &mut current_path, &mut out_map);
                }
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    current_path.clear();
                    current_path.push_str(&key);
                    self.json_to_map(val, &mut current_path, &mut out_map);
                }
            }
            _ => {}
        }

        req.form.values.extend(out_map);

        Ok(())
    }

    fn json_to_map(
        &self,
        value: Value,
        current_path: &mut String,
        out_map: &mut HashMap<String, String>,
    ) {
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
                let base_len = current_path.len();
                for (i, item) in arr.into_iter().enumerate() {
                    current_path.truncate(base_len);
                    let _ = write!(current_path, "[{}]", i);
                    self.json_to_map(item, current_path, out_map);
                }
                current_path.truncate(base_len);
            }
            Value::Object(obj) => {
                let base_len = current_path.len();
                for (key, val) in obj {
                    current_path.truncate(base_len);
                    let _ = write!(current_path, "[{}]", key);
                    self.json_to_map(val, current_path, out_map);
                }
                current_path.truncate(base_len);
            }
        }
    }
}

impl Hook for FormHook {
    async fn before(&self, mut req: Request, res: Response, next: Next) -> Response {
        let _ = self.parse(&mut req).await;
        next.handle(req, res)
    }

    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        next.handle(req, res)
    }
}