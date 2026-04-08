use std::time::Duration;

use anyhow::Result;
use cookie::Cookie;
use serde::{Deserialize, Serialize};
use cookie::time::{Duration as DurationCookie, OffsetDateTime};

use crate::session::cookie::utils::parse_encrypted_raw_cookie;
use crate::session::{Session, SessionManager};
use crate::response::Response;
use crate::request::Request;

pub(crate) mod utils;

use crate::utils::{
    Values,
    encrypt::encrypt,
    string::string_fixed_length,
    cookie::cookie_parse
};

pub struct SessionCookieManager {
    expires: Duration,
    cookie_name: String,
    encryption_key: String,
}

impl SessionCookieManager {
    pub fn new(expires: Duration, cookie_name: &str, encryption_key: &str) -> Self {
        return Self {
            expires: expires,
            cookie_name: cookie_name.to_owned(),
            encryption_key: string_fixed_length(encryption_key, 32),
        };
    }
}

#[derive(Debug, Default)]
pub(crate) struct SessionCookie {
    pub(crate) values: Values,
    pub(crate) errors: Values,
    pub(crate) old: Values,
    pub(crate) new_old: Values,
    pub(crate) new_errors: Values,
}

impl SessionCookie {
    pub(crate) fn new(values: Values, errors: Values, old: Values) -> Self {
        return Self {
            values: values,
            errors: errors,
            old: old,
            new_old: Values::new(),
            new_errors: Values::new(),
        }
    }
}

#[deprecated]
pub fn new_session_manager(expires: Duration, cookie_name: &str, encryption_key: &str) -> impl SessionManager {
    return SessionCookieManager {
        expires: expires,
        cookie_name: String::from(cookie_name),
        encryption_key: string_fixed_length(encryption_key, 32),
    };
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CookieStorage {
    pub values: Values,
    pub errors: Values,
    pub old: Values,
}

// TODO: refactor
impl SessionManager for SessionCookieManager {
    fn setup<'a>(&mut self, req: &'a mut Request, _res: &'a mut Response) -> Result<()> {
        req.session = match cookie_parse(req.header("cookie")).unwrap().get(&self.cookie_name) {
            Some(raw) => Some(Box::new(parse_encrypted_raw_cookie(self.encryption_key.to_owned(), raw).unwrap())),
            None => Some(Box::new(SessionCookie::default())),
        };

        return Ok(())
    }

    fn teardown<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {        
        unsafe {
            let ptr = req.session.as_mut().unwrap() as *mut Box<dyn Session + 'static> as usize;
            let session = &mut **(ptr as *mut Box<SessionCookie>);
            
            let data = serde_json::to_string(&CookieStorage {
                values: session.values.clone(),
                errors: res.errors.clone(),
                old: res.old.clone(),
            });

            let payload = encrypt(self.encryption_key.as_str(), data.unwrap().as_str()).unwrap();
            let mut cookie = Cookie::new(self.cookie_name.clone(), payload);

            cookie.set_expires(OffsetDateTime::now_utc() + DurationCookie::seconds(self.expires.as_secs().try_into().unwrap()));

            res.header("Set-Cookie", &cookie.to_string());

            return Ok(());
        };
    }
}

impl Session for SessionCookie {
    fn values(&mut self) -> Values {
        return self.values.clone();
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
        return self.values.get(key).map(|v| String::from(v)).unwrap_or(String::new());
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
        return self.errors.get(key).map(|e| String::from(e)).unwrap_or(String::new());
    }

    fn remove_error(&mut self, key: &str) {
        self.errors.remove(key);
    }

    fn set_old(&mut self, values: Values) {
        for (key, value) in values {
            self.new_old.insert(key, value);
        }
    }

    fn old_values(&mut self) -> Values {
        return self.old.clone();
    }

    fn old(&mut self, key: &str) -> String {
        return self.old.get(key).or(Some(&String::new())).unwrap().to_string();
    }
}
