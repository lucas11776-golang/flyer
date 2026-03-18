use anyhow::Result;

use crate::session::cookie::{CookieStorage, SessionCookie};
use crate::utils::Values;
use crate::utils::encrypt::decrypt;

pub(crate) fn parse_raw_cookie(encryption_key: String, raw_cookie: Option<&String>) -> Result<SessionCookie> {
    if raw_cookie.is_none() {
        return Ok(SessionCookie::new(Values::new(), Values::new(), Values::new()));
    }

    let payload = decrypt(&encryption_key, raw_cookie.unwrap());

    if payload.is_err() {
        return Ok(SessionCookie::new(Values::new(), Values::new(), Values::new()));
    }

    let result = serde_json::from_str::<CookieStorage>(&payload.unwrap());

    if result.is_err() {
        return Ok(SessionCookie::new(Values::new(), Values::new(), Values::new()));
    }

    let storage = result.unwrap();

    return Ok(SessionCookie::new(storage.values, storage.errors, storage.old));
}