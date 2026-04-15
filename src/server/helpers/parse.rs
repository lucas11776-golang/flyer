

use crate::{cookies::Cookie, response::Response};

pub fn http_1_parse(response: &mut Response, cookies: Option<&mut Vec<Cookie>>) -> Vec<u8> {
    let mut res = Vec::new();

    res.extend_from_slice(format!("HTTP/1.0 {} OK\r\n", response.status_code).as_bytes());

    for (k, v) in &response.headers {
        res.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
    }

    if let Some(cookies) = cookies {
        for cookie in cookies {
            res.extend_from_slice(format!("Set-Cookie: {}\r\n", cookie.parse()).as_bytes());
        }
    }

    res.extend_from_slice(b"\r\n");
    res.extend_from_slice(&response.body);

    return res;
}