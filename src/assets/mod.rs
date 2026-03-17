use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Result},
};

use mime_guess::from_path;

use crate::{
    request::Request,
    response::Response,
    utils::timestamp
};


#[derive(Clone)]
pub(crate) struct Asset {
    pub size: usize,
    pub expires: u128,
    pub data: Vec<u8>,
    pub content_type: String,
}

pub(crate) type Cache = HashMap<String, Asset>;

pub(crate) struct Assets {
    path: String,
    expires: u128,
    max_size: usize,
    cache: Cache,
}

// TODO: Refactor
impl Assets {
    pub fn new(path: String, max_size_kilobytes_cache_size: usize, expires_in_seconds: u128) -> Self {
        return Self {
            path: path.trim_end_matches("/").to_owned(),
            max_size: max_size_kilobytes_cache_size * 1000,
            expires: expires_in_seconds,
            cache: Cache::new(),
        }
    }

    pub fn handle<'a>(&mut self, req: &'a mut Request, resp: &'a mut Response) -> Result<()> {
        if req.path.trim_matches('/').is_empty() {
            return Ok(());
        }

        let name = format!("{}/{}", self.path, req.path.trim_start_matches("/"));

        match self.cache.get(&name) {
            Some(asset) => {
                resp.body(&asset.data)
                    .status_code(200)
                    .header("Content-Type", &asset.content_type);
            },
            None => {
                match self.read_asset(&name) {
                    Some(asset) => {
                        resp.body(&asset.data)
                            .header("Content-Type", &asset.content_type)
                            .status_code(200);
                    },
                    None => { },
                };
            },
        };

        return Ok(());
    }

    fn read_asset(&mut self, filename: &str) -> Option<Asset> {
        return match File::open(filename) {
            Ok(mut file) => {
                let mut data= String::new();

                let asset = Asset {
                    size: file.read_to_string(&mut data).unwrap(),
                    expires: timestamp().unwrap() + (1000 * self.expires),
                    data: data.into_bytes(),
                    content_type: self.get_content_type(filename),
                };

                if asset.size <= self.max_size {
                    if let Some(asset) = self.cache.insert(filename.to_string(), asset.clone()) {
                        return Some(asset)
                    }
                }

                return Some(asset);
            },
            Err(_) => None,
        };
    }

    fn get_content_type(&self, filename: &str) -> String {
        return from_path(filename.split("/").last().unwrap_or("")).first_or_octet_stream().to_string();
    }

}