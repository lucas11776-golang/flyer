use std::io::Result;

use crate::{cookie::Cookie, response::Response};

pub fn parse(response: &mut Response, cookies: Option<&mut Vec<Cookie>>) -> Result<String> {
    let mut res: Vec<String> = vec![format!("HTTP/1.0 {} {}", response.status_code, "OK")];

    for (k, v) in response.headers.clone() {
        res.push(format!("{}: {}", k, v));
    }

    res.push(format!("Content-Length: {}", response.body.len()));
    
    if let Some(cookies) = cookies {
        for cookie in cookies {
            res.push(format!("Set-Cookie: {}", cookie.parse()));
        }
    }

    res.push(format!("\r\n{}", String::from_utf8(response.body.clone()).unwrap()));

    println!("{}", res.join("\r\n"));

    return Ok(res.join("\r\n"));
}