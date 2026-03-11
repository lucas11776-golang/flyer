use std::time::Duration;

use cookie::{
    Cookie as OrgCookie,
    time::{Duration as CookieDuration, OffsetDateTime}
};

use crate::utils::Values;

pub struct Cookie {
    name: String,
    value: String,
    expires: Option<Duration>,
    max_age: Option<Duration>,
    domain: Option<String>,
    path: Option<String>,
    secure: Option<bool>,
    http_only: Option<bool>,

}

pub struct Cookies {
    pub(crate) cookies: Values,
    pub(crate) new_cookie: Vec<Cookie>,
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

    pub fn set_http_only(&mut self, value: bool) -> &mut Self {
        self.http_only = Some(value);

        return self;
    }

    pub(crate) fn parse(&mut self) -> String {
        let mut cookie = OrgCookie::new(self.name.to_string(), self.value.to_string());

        if self.expires.is_some() {
            cookie.set_expires(OffsetDateTime::now_utc() + CookieDuration::seconds(self.expires.unwrap().as_secs().try_into().unwrap()));
        }

        if self.max_age.is_some() {
            cookie.set_max_age(CookieDuration::new(self.max_age.unwrap().as_secs().try_into().unwrap(), 0));
        }

        if self.domain.is_some() {
            cookie.set_domain(self.domain.as_ref().unwrap());
        }

        if self.domain.is_some() {
            cookie.set_domain(self.domain.as_ref().unwrap());
        }

        if self.secure.is_some() && self.secure.unwrap() {
            cookie.set_secure(true);
        }

        if self.http_only.is_some() && self.http_only.unwrap() {
            cookie.set_http_only(true);
        }

        return cookie.to_string();
    }
}

impl Cookies {
    pub fn new(cookies: Values) -> Self {
        return Self {
            cookies: cookies,
            new_cookie: vec![],
        }
    }

    pub fn get(&mut self, name: &str) -> String {
        let cookie = self.cookies.get(name);

        if cookie.is_none() {
            return String::new()
        }

        return cookie.unwrap().to_owned();
    }

    pub fn set(&mut self, name: &str, value: &str) -> &mut Cookie {
        let idx = self.new_cookie.len();

        self.new_cookie.push(Cookie::new(name, value));

        return &mut self.new_cookie[idx];
    }
}
