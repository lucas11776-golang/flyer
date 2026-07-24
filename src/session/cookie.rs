use std::time::Duration;

use aes_gcm::{
    AeadCore,
    Aes256Gcm,
    Key,
    Nonce,
    aead::{Aead, KeyInit, OsRng}
};
use anyhow::Result;
use base64::{Engine, engine::general_purpose};

use crate::{
    hooks::Hook,
    request::Request,
    response::Response,
    routing::next::Next,
    session::Session
};

pub struct CookieSession {
    cookie_name: String,
    encryption_key: String,
    duration: Duration,
}

impl CookieSession {
    pub fn new(cookie_name: impl Into<String>, encryption_key: impl Into<String>, expires: Duration) -> Self {
        return Self {
            cookie_name: cookie_name.into(),
            encryption_key:  Self::string_fixed_length(&encryption_key.into(), 32),
            duration: expires,
        };
    }
}

impl Hook for CookieSession {
    async fn before(&self, mut req: Request, res: Response, next: Next) -> Response {
        let hash = req.cookie(self.cookie_name.clone());

        if hash.is_empty() {
            return next.handle(req, res);
        }

        let result = self.decrypt(&hash);

        if result.is_err() {
            return next.handle(req, res);
        }

        let result = serde_json::from_str::<Session>(&result.unwrap());

        if result.is_err() {
            return next.handle(req, res);
        }

        req.session = result.unwrap();

        return next.handle(req, res);
    }
    
    async fn after(&self, req: Request, mut res: Response, next: Next) -> Response {
        let data = serde_json::to_string(&res.session)
            .unwrap();
        let payload = self
            .encrypt(&data)
            .unwrap();
        let cookie = res
            .set_cookie(self.cookie_name.clone(), payload)
            .set_expires(self.duration)
            .set_same_site(crate::cookies::SameSite::Lax)
            .set_path("/")
            .parse();

        return next.handle(req, res.set_header("Set-Cookie", cookie));
    }
}

impl CookieSession {
    pub fn string_fixed_length(input: &str, length: usize) -> String {
        return match input.len() < length {
            true => format!("{:<width$}", input, width = length),
            false => input.chars().take(length).collect(),
        };
    }

    pub fn encrypt(&self, data: &str) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(self.encryption_key.as_bytes());
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(OsRng);
        let cipher_text = cipher.encrypt(&nonce, data.as_bytes()).unwrap();
        let mut combined = nonce.to_vec();

        combined.extend_from_slice(&cipher_text);

        let mut buffer = String::new();

        general_purpose::STANDARD.encode_string(combined, &mut buffer);

        return Ok(buffer);
    }

    pub fn decrypt(&self, hash: &str) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(self.encryption_key.as_bytes());
        let cipher = Aes256Gcm::new(key);
        let combined = general_purpose::STANDARD.decode(hash);

        if combined.is_err() {
            return Ok(String::new());
        }

        let session = combined.unwrap();
        let (nonce_bytes, ciphertext) = session.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();

        return String::from_utf8(plaintext).map_err(|err| err.into());
    }
}