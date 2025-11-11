use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Result},
};

use crate::{request::Request, response::Response, utils::timestamp};

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

// TODO: refactor cache, implement expiring cache
impl Assets {
    pub fn new(path: String, max_size_kilobytes: usize, expires_in_seconds: u128) -> Self {
        return Self {
            path: path.trim_end_matches("/").to_owned(),
            expires: expires_in_seconds,
            max_size: max_size_kilobytes,
            cache: Cache::new(),
        }
    }

    pub fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        let name = format!("{}/{}", self.path, req.path.trim_start_matches("/"));
        let cached_asset = self.cache.get(&name);

        if cached_asset.is_some() {
            let asset = cached_asset.unwrap();


            // println!("EXPIRES, {}", asset.expires - timestamp().unwrap() );

            if timestamp().unwrap() > cached_asset.unwrap().expires {
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
            // remove in cache if big fix...
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
            expires: timestamp().unwrap(),
            data: content.into(),
        });

        let asset = self.cache.get(&name).unwrap();

        return Ok(Some(asset))
    }

}