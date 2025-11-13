use std::io::Result;

use cookie::Cookie;

use crate::utils::Values;

pub fn cookie_parse<'a>(raw_cookie: String) -> Result<Values> {
    let mut values = Values::new();

    for result in Cookie::split_parse(raw_cookie) {
        let cookie = result.unwrap();

        values.insert(cookie.name().to_string(), cookie.value().to_string());
    }

    return Ok(values);
}