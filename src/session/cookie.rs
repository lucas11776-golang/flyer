use std::any::Any;
use std::io::Result;
use std::time::Duration;

use cookie::Cookie;
use serde::{Deserialize, Serialize};
use cookie::time::{Duration as DurationCookie, OffsetDateTime};

use crate::session::{Session, SessionManager};
use crate::response::Response;
use crate::request::Request;

use crate::utils::{
    Values,
    encrypt::{decrypt, encrypt},
    string::string_fixed_length,
    cookie::cookie_parse
};

pub struct SessionCookieManager {
    expires: Duration,
    cookie_name: String,
    encryption_key: String,
}


pub(crate) struct SessionCookie {
    pub(crate) values: Values,
    pub(crate) errors: Values,
    pub(crate) new_errors: Values,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CookieStorage {
    pub values: Values,
    pub errors: Values,
}

pub(crate) fn new_session_cookie(values: Values, errors: Values) -> SessionCookie {
    return SessionCookie {
        values: values,
        errors: errors,
        new_errors: Values::new(),
    }
}

pub fn new_session_manager(expires: Duration, cookie_name: &str, encryption_key: &str) -> impl SessionManager {
    return SessionCookieManager {
        expires: expires,
        cookie_name: cookie_name.to_owned(),
        encryption_key: string_fixed_length(encryption_key, 32),
    };
}



pub(crate) fn parse_raw_cookie(encryption_key: String, raw_cookie: Option<&String>) -> Result<SessionCookie> {
    if raw_cookie.is_none() {
        return Ok(new_session_cookie(Values::new(), Values::new()));
    }

    let payload = decrypt(&encryption_key, raw_cookie.unwrap());

    if payload.is_err() {
        return Ok(new_session_cookie(Values::new(), Values::new()));
    }

    let storage: CookieStorage = serde_json::from_str(&payload.unwrap()).unwrap();

    return Ok(new_session_cookie(storage.values, storage.errors));
}

impl SessionManager for SessionCookieManager {
    fn setup<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        let cookies = cookie_parse(req.header("cookie")).unwrap();
        let raw_cookie = cookies.get(&self.cookie_name);
        req.session = Some(Box::new(parse_raw_cookie(self.encryption_key.to_owned(), raw_cookie).unwrap()));

        return Ok((req, res));
    }

    // TODO: set domain as global maybe have configuration struct in `new_session_manager`.
    fn teardown<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {        
        // TODO: do not like is - (working with unsafe...)
        unsafe {
            
            // TODO: loosening performance here in downcast ref.
            let session = (req.session.as_ref().unwrap() as &dyn Any).downcast_ref_unchecked::<Box<SessionCookie>>();

            let data = serde_json::to_string(&CookieStorage {
                values: session.values.clone(),
                errors: session.new_errors.clone(),
            });

            // TODO: also encrypt will take time.
            let payload = encrypt(self.encryption_key.as_str(), data.unwrap().as_str()).unwrap();
            let mut cookie = Cookie::new(self.cookie_name.clone(), payload);

            cookie.set_expires(OffsetDateTime::now_utc() + DurationCookie::seconds(self.expires.as_secs().try_into().unwrap()));

            return Ok((req, res.header("Set-Cookie", &cookie.to_string())));
        };
    }
}


impl Session for SessionCookie {
    fn values(&mut self) -> Values {
        todo!()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_owned(), value.to_owned());
    }

    fn set_values(&mut self, values: Values) {
        for (key, value) in values {
            self.set(key.as_str(), value.as_str());
        }
    }

    fn get(&mut self, key: &str) -> String {
        let value = self.values.get(key);

        if value.is_none() {
            return String::new();
        }

        return value.unwrap().to_owned();
    }

    fn remove(&mut self, key: &str) {
        self.values.remove(key);
    }

    fn errors(&mut self) -> Values {
        return self.errors.clone();
    }

    fn set_error(&mut self, key: &str, value: &str) {
        self.new_errors.insert(key.to_owned(), value.to_owned());
    }

    fn set_errors(&mut self, errors: Values) {
        for (key, value) in errors {
            self.set_error(key.as_str(), value.as_str());
        }
    }

    fn get_error(&mut self, key: &str) -> String {
        let error = self.errors.get(key);

        if error.is_none() {
            return String::new();
        }
        
        return error.unwrap().to_owned();
    }

    fn remove_error(&mut self, key: &str) {
        self.errors.remove(key);
    }
}
