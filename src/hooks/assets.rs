use std::{
    path::{Component, Path, PathBuf},
    time::Duration,
};

use bytes::Bytes;
use mime_guess::from_path;
use moka::sync::Cache;

use crate::{
    hooks::Hook,
    request::Request,
    response::{HTTP_NOT_FOUND, HTTP_OK, Response},
    routing::next::Next,
};

#[derive(Clone)]
pub(crate) struct Asset {
    pub data: Bytes,
    pub content_type: String,
}

pub struct AssetsHook {
    base_dir: PathBuf,
    max_file_size_bytes: usize,
    cache: Cache<String, Asset>,
}

impl AssetsHook {
    pub fn new(directory: impl AsRef<Path>, expires_in: Duration, max_cache_size_kilobytes: u64) -> Self {
        let base_dir = directory
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| directory.as_ref().to_path_buf());

        let max_file_size_bytes = (max_cache_size_kilobytes as usize).saturating_mul(1024);

        let mut builder = Cache::builder()
            .max_capacity(100_000_000) // 100 MB max total cache memory capacity
            .weigher(|_key, asset: &Asset| asset.data.len() as u32);

        if !expires_in.is_zero() {
            builder = builder.time_to_live(expires_in);
        }

        Self {
            base_dir,
            max_file_size_bytes,
            cache: builder.build(),
        }
    }

    fn get_safe_path(&self, req_path: &str) -> Option<PathBuf> {
        let mut path = self.base_dir.clone();

        for component in Path::new(req_path).components() {
            match component {
                Component::Normal(c) => path.push(c),
                Component::RootDir | Component::CurDir => continue,
                _ => return None,
            }
        }

        if path.is_file() {
            return Some(path);
        }

        None
    }

    fn guess_content_type(path: &Path) -> String {
        from_path(path)
            .first_or_octet_stream()
            .to_string()
    }
}

impl Hook for AssetsHook {
    async fn before(&self,req: Request, res: Response, next: Next) -> Response {
        next.handle(req, res)
    }

    async fn after(&self, req: Request, res: Response, next: Next) -> Response {
        if res.status_code != HTTP_NOT_FOUND {
            return next.handle(req, res);
        }

        let path = req.path();

        if let Some(asset) = self.cache.get(&path) {
            return res
                .status_code(HTTP_OK)
                .body(asset.data)
                .set_header("Content-Type", asset.content_type);
        }

        if let Some(file_path) = self.get_safe_path(&path) {
            if let Ok(file_bytes) = tokio::fs::read(&file_path).await {
                let data = Bytes::from(file_bytes);
                let content_type = Self::guess_content_type(&file_path);

                let asset = Asset {
                    data: data.clone(),
                    content_type: content_type.clone(),
                };

                if data.len() < self.max_file_size_bytes {
                    self.cache.insert(path.to_string(), asset);
                }

                return res
                    .status_code(HTTP_OK)
                    .body(data)
                    .set_header("Content-Type", content_type);
            }
        }

        next.handle(req, res)
    }
}