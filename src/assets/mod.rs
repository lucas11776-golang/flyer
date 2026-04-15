use std::{fs::File, io::Read};

use mime_guess::from_path;

use crate::{assets::cache::{Asset, LruCache}, request::Request, response::Response, utils::timestamp};

pub(crate) mod cache;

pub(crate) struct Assets {
    path: String,
    size: usize,
    cache: LruCache,
    expires: u128,
}

impl Assets {
    pub fn new(path: String, max_size_kilobytes_cache_size: usize, expires_in_seconds: u128) -> Self {
        return Self {
            path: path,
            size: max_size_kilobytes_cache_size,
            cache: LruCache::new(100),
            expires: expires_in_seconds
        }
    }

    pub fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) {
        if req.path.trim_matches('/').is_empty() {
            return;
        }

        if let Some(asset) = self.get_asset(format!("{}/{}", self.path, req.path.trim_start_matches("/"))) {
            res.body(&asset.data)
                .header("Content-Type", &asset.content_type)
                .status_code(200);
        }
    }

    fn get_asset(&mut self, filename: String) -> Option<Asset> {
        return match self.cache.get(&filename, self.expires) {
            Some(asset) => {
                Some(asset.clone())
            },
            None => {
                match self.read_asset(&filename) {
                    Some(asset) => {
                        if self.size <= asset.size {
                            self.cache.insert(String::from(filename), asset.clone());
                        }

                        return Some(asset);
                    },
                    None => {
                        None
                    },
                }
            },
        }
    }

    fn read_asset(&self, filename: &str) -> Option<Asset> {
        return match File::open(filename) {
            Ok(mut file) => {
                let mut data= String::new();

                return Some(Asset {
                    size: file.read_to_string(&mut data).unwrap(),
                    expires: timestamp().unwrap() + (1000 * self.expires),
                    data: data.into_bytes(),
                    content_type: self.get_content_type(filename),
                });
            },
            Err(_) => {
                None
            },
        };
    }

    fn get_content_type(&self, filename: &str) -> String {
        return from_path(filename.split("/").last().unwrap_or("")).first_or_octet_stream().to_string();
    }
}