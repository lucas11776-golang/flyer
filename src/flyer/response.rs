use std::io::Result;

use serde::Serialize;

use crate::flyer::request::{Headers};

pub struct Response {
    status_code: i32,
    headers: Headers,
    body: Vec<u8>,
}

pub fn new_response() -> Response {
    return Response {
        status_code: 200,
        headers: Headers::new(),
        body: vec![],
    }
}

pub fn parse(response: &mut Response) -> Result<String> {
    let mut res: Vec<String> = vec![format!("HTTP/1.0 {} {}", response.status_code, "OK")];

    for ele in response.headers.clone() {
        res.push(format!("{}: {}", ele.0, ele.1));
    }

    res.push(format!("Content-Length: {}", response.body.len()));
    res.push(format!("\r\n{}", String::from_utf8(response.body.clone()).unwrap()));

    return Ok(res.join("\r\n"));
}

impl Response {
    pub fn status_code(&mut self, code: i32) -> &mut Response {
        self.status_code = code;
        
        return self;
    }

    pub fn header(&mut self, key: String, value: String) -> &mut Response {
        self.headers.insert(key, value);

        return self;
    }

    pub fn headers(&mut self, headers: Headers) -> &mut Response {
        for ele in headers {
            self.header(ele.0, ele.1);
        }

        return self;
    }

    pub fn body(&mut self, body: &[u8]) -> &mut Response {
        self.body = body.to_vec();

        return self;
    }

    // where -> specify the type of data that is allowed in T.
    pub fn json<T>(&mut self, json: &T) -> &mut Response where T: ?Sized + Serialize, {
        return self.header("Content-Type".to_owned(), "application/json".to_owned())
            .body(serde_json::to_string(json).unwrap().as_bytes());
    }

    pub fn html(&mut self, html: String) -> &mut Response {
        return self.header("Content-Type".to_string(), "text/html".to_owned())
            .body(html.as_bytes());
    }
}