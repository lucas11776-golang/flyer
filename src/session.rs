use aes_gcm::{Aes256Gcm, Key};
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

use crate::request::Request;
use crate::response::Response;
use crate::{Session, SessionManager, Values};

lazy_static! {
    static ref SECRET_KEY: Key<Aes256Gcm> = Key::<Aes256Gcm>::from_slice(b"abc").clone();
}

pub struct DefaultSessionManager {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CookieSession {
    pub(crate) values: Values,
}

impl SessionManager for DefaultSessionManager {
    // pub fn new(token: &str) -> Self {
    //     DefaultSessionManager{
    //         token: token.to_string()
    //     }
    // }

    fn handle<'a>(&self, req: &'a mut Request, res: &'a mut Response) -> Box<dyn Session + 'a> {
        return Box::new(CookieSession{
            values: Values::new()
        });
    }
}

impl Session for CookieSession {
    fn set(&self, key: &str, value: &str) {

    }
    
    fn get(&self, key: &str) -> String {
        return "".to_string();
    }
}

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub max_age: i64,
    pub path: String,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}
