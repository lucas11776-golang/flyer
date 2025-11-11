
use std::{collections::HashMap, fs::File, io::Read};

use crate::{request::Request, response::Response};

pub(crate) struct Asset {
    pub expires: u64,
    pub data: Vec<u8>,
}

pub(crate) type Cache = HashMap<String, Asset>;

pub(crate) struct Assets {
    path: String,
    expires: u64,
    max_size: u64,
    cache: Cache,
}

impl Assets {
    pub fn new(path: String, max_size: u64, expires: u64) -> Self {
        return Self {
            path: path.trim_end_matches("/").to_owned(),
            expires: expires,
            max_size: max_size,
            cache: Cache::new(),
        }
    }

    pub async fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> (&'a mut Request, &'a mut Response) {
        let file = File::open(format!("{}/{}", self.path, req.path.trim_start_matches("/")));

        if !file.is_ok() {
            return (req, res);
        }

        let mut content = String::new();
        let reading = file.unwrap().read_to_string(&mut content);

        if !reading.is_ok() {
            return (req, res);
        }
        
        reading.unwrap();

        return (req, res.body(content.as_bytes()).status_code(200));
    }
}