use std::{collections::HashMap, time::Duration};

use crate::utils::Values;

pub enum SameSite {
    Strict,
    None,
    Lax,
}

pub struct Cookie {
    name: String,
    value: String,
    expires: Option<Duration>,
    max_age: Option<Duration>,
    domain: Option<String>,
    path: Option<String>,
    secure: Option<bool>,
    http_only: Option<SameSite>,

}

pub struct Cookies {
    pub(crate) cookies: Values,
    pub(crate) new_cookie: HashMap<String, Cookie>,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        return Self {
            name: name.to_string(),
            value: value.to_string(),
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: None,
            http_only: None,
        }
    }

    pub fn set_name(&mut self, value: &str) -> &mut Self {
        self.name = value.to_string();

        return self;
    }

    pub fn set_value(&mut self, value: &str) -> &mut Self {
        self.value = value.to_string();

        return self;
    }

    pub fn set_expires(&mut self, duration: Duration) -> &mut Self {
        self.expires = Some(duration);

        return self;
    }

    pub fn set_max_age(&mut self, duration: Duration) -> &mut Self {
        self.max_age = Some(duration);

        return self;
    }

    pub fn set_domain(&mut self, value: &str) -> &mut Self {
        self.domain = Some(value.to_string());

        return self;
    }

    pub fn set_path(&mut self, value: &str) -> &mut Self {
        self.path = Some(value.to_string());

        return self;
    }

    pub fn set_secure(&mut self, value: bool) -> &mut Self {
        self.secure = Some(value);

        return self;
    }

    pub fn set_http_only(&mut self, value: SameSite) -> &mut Self {
        self.http_only = Some(value);

        return self;
    }
}

impl Cookies {
    pub fn new(cookies: Values) -> Self {
        return Self {
            cookies: cookies,
            new_cookie: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &str) -> String {
        let cookie = self.cookies.get(name);

        if cookie.is_none() {
            return String::new()
        }

        return cookie.unwrap().to_owned();
    }

    pub fn set(&mut self, name: &str, value: &str) -> &Cookie {
        self.new_cookie.insert(name.to_owned(), Cookie::new(name, value));

        return self.new_cookie.get(name).unwrap();
    }
}
