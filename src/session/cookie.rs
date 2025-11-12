use std::any::Any;
use std::io::Result;
use std::time::Duration;

use cookie::Cookie;
use serde::{Deserialize, Serialize};
use tera::Value;

use crate::session::{Session, SessionManager};
use crate::response::Response;
use crate::request::Request;
use crate::utils::Values;
use crate::utils::encrypt::{decrypt, encrypt};
use crate::utils::string::string_fixed_length;

use cookie::time::{Duration as DurationCookie, OffsetDateTime};

pub fn new_session_manager(expires: Duration, cookie_name: &str, encryption_key: &str) -> impl SessionManager {
    return SessionCookieManager {
        expires: expires,
        cookie_name: cookie_name.to_owned(),
        encryption_key: string_fixed_length(encryption_key, 32),
    };
}

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


pub(crate) fn new_session_cookie(values: Values, errors: Values) -> SessionCookie {
    return SessionCookie {
        values: values,
        errors: errors,
        new_errors: Values::new(),
    }
}


pub fn cookie_parse<'a>(raw_cookie: String) -> Result<Values> {
    let mut values = Values::new();

    for result in Cookie::split_parse(raw_cookie) {
        let cookie = result.unwrap();

        values.insert(cookie.name().to_string(), cookie.value().to_string());
    }

    return Ok(values);
}


#[derive(Serialize, Deserialize)]
pub(crate) struct CookieStorage {
    pub values: Values,
    pub errors: Values,
}


impl SessionManager for SessionCookieManager {
    fn setup<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {
        let cookies = cookie_parse(req.header("cookie")).unwrap();
        let raw_session = cookies.get(&self.cookie_name);
        let session: SessionCookie;

        if raw_session.is_some() {
            let data = decrypt(&self.encryption_key, raw_session.unwrap());

            if data.is_err() {
                session = new_session_cookie(Values::new(), Values::new());
            } else {
                let storage: CookieStorage = serde_json::from_str(&data.unwrap()).unwrap();

                session = new_session_cookie(storage.values, storage.errors);
            }
        } else {
            session = new_session_cookie(Values::new(), Values::new());
        }

        req.session = Some(Box::new(session));

        return Ok((req, res));
    }

    // TODO: set domain as global maybe have configuration struct in `new_session_manager`.
    // TODO: performance drop almost 2x.
    fn teardown<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> Result<(&'a mut Request, &'a mut Response)> {        
        // TODO: do not like is - (working with unsafe...)
        unsafe {
            let session = (req.session.as_ref().unwrap() as &dyn Any).downcast_ref_unchecked::<Box<SessionCookie>>();
            let data = serde_json::to_string(&CookieStorage {
                values: session.values.clone(),
                errors: session.new_errors.clone(),
            });

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

    fn set_values(&mut self, values: Values) -> Values {
        todo!()
    }

    fn get(&mut self, key: &str) -> String {
        let value = self.values.get(key);

        if value.is_none() {
            return String::new();
        }

        return value.unwrap().to_owned();
    }

    fn remove(&mut self, key: &str) {
        todo!()
    }

    fn errors(&mut self) -> Values {
        todo!()
    }

    fn set_errors(&mut self, errors: Values) -> Values {
        todo!()
    }

    fn set_error(&mut self, key: &str, value: &str) {
        todo!()
    }

    fn get_error(&mut self, key: &str) {
        todo!()
    }

    fn remove_error(&mut self, key: &str) {
        todo!()
    }
}
