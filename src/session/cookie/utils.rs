use anyhow::Result;

use crate::session::cookie::{CookieStorage, SessionCookie};
use crate::utils::encrypt::decrypt;

pub(crate) fn parse_encrypted_raw_cookie(encryption_key: String, raw_encrypted_cookie: &str) -> Result<SessionCookie> {
    if let Ok(payload) = decrypt(&encryption_key, raw_encrypted_cookie) {
        if let Ok(storage) = serde_json::from_str::<CookieStorage>(&payload) {
            return Ok(SessionCookie::new(storage.values, storage.errors, storage.old));
        }
    }

    return Ok(SessionCookie::default());
}