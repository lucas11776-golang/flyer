use std::io::Result;

use cookie::Cookie;

use crate::utils::Values;

pub fn cookie_parse<'a>(raw: String) -> Result<Values> {
    let mut cookies = Values::new();

    for cookie in Cookie::split_parse(raw) {
        if let Ok(c) = cookie {
            cookies.insert(c.name().to_string(), c.value().to_string());
        }
    }

    return Ok(cookies);
}