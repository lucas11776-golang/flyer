use std::time::{SystemTime, Duration};

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use serde::{Deserialize, Serialize};

use base64::{engine::general_purpose, Engine as _};
use lazy_static::lazy_static;
use uuid::Uuid;

use crate::request::Request;
use crate::response::Response;
use crate::Values;

lazy_static! {
    static ref SECRET_KEY: Key<Aes256Gcm> = Key::<Aes256Gcm>::from_slice(b"abc").clone();
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub values: Values,
    pub expires_at: SystemTime,
}

pub struct SessionManager {
    pub(crate) token: String,
    pub(crate) expires: i64, 
}

impl SessionManager {
    pub(crate) fn handle<'a>(req: &'a mut Request) {
        
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            values: Values::new(),
            expires_at: SystemTime::now() + Duration::from_secs(60 * 60)
        }
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        return Some(self.get(key).unwrap())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// Encrypt a session ID into a token
fn encrypt(session_id: &str) -> Option<String> {
    let cipher = Aes256Gcm::new(&SECRET_KEY);

    let nonce = Nonce::from_slice(b"unique_nonce12");

    cipher.encrypt(nonce, session_id.as_bytes())
        .ok()
        .map(|ciphertext| {
            let mut token = nonce.to_vec();
            token.extend(ciphertext);
            general_purpose::STANDARD.encode(token)
        })
}

/// Decrypt a token back into session ID
fn decrypt(token: &str) -> Option<String> {
    let cipher = Aes256Gcm::new(&SECRET_KEY);

    let data = general_purpose::STANDARD.decode(token).ok()?;
    if data.len() < 12 {
        return None;
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher.decrypt(nonce, ciphertext).ok()
        .and_then(|plaintext| String::from_utf8(plaintext).ok())
}

pub fn load_or_create_session(req: &mut Request, res: &mut Response) {
    match req.headers.get("token") {
        Some(token) => {

        },
        None => {
            req.session = Some(Session::new())
        },
    }
}

