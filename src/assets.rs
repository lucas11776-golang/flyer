use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Result},
};

use crate::{
    request::Request,
    response::Response,
    utils::timestamp
};

pub(crate) struct Asset {
    pub size: usize,
    pub expires: u128,
    pub data: Vec<u8>,
}

pub(crate) type Cache = HashMap<String, Asset>;

pub(crate) struct Assets {
    path: String,
    expires: u128,
    max_size: usize,
    cache: Cache,
}

impl Assets {
    pub fn new(path: String, max_size_kilobytes: usize, expires_in_seconds: u128) -> Self {
        return Self {
            path: path.trim_end_matches("/").to_owned(),
            max_size: max_size_kilobytes * 1000,
            expires: expires_in_seconds,
            cache: Cache::new(),
        }
    }

    pub fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        let name = format!("{}/{}", self.path, req.path.trim_start_matches("/"));
        let cached_asset = self.cache.get(&name);

        if cached_asset.is_some() {
            let asset = cached_asset.unwrap();

            if  asset.expires > timestamp().unwrap() {
                return Ok((req, res.body(&asset.data).status_code(200)));
            }
        }

        let cached_asset = self.read_asset(name.clone()).unwrap();

        if !cached_asset.is_some() {
            return Ok((req, res));
        }

        let asset = cached_asset.unwrap();

        res.body(&asset.data).status_code(200);


        if asset.size > self.max_size.clone() {
            self.cache.remove(&name.clone());
        } 

        return Ok((req, res));
    }


    fn read_asset(&mut self, name: String) -> Result<Option<&Asset>> {
        let file = File::open(name.clone());

        if !file.is_ok() {
            return Ok(None);
        }

        let mut content = String::new();
        let reading = file.unwrap().read_to_string(&mut content);

        if !reading.is_ok() {
            return Ok(None);
        }

        self.cache.insert(name.clone(), Asset {
            size: reading.unwrap(),
            expires: timestamp().unwrap() + (1000 * self.expires),
            data: content.into(),
        });

        let asset = self.cache.get(&name).unwrap();

        return Ok(Some(asset))
    }

}