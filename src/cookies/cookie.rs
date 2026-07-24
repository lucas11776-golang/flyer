use std::time::Duration;

use cookie::time::OffsetDateTime;

use crate::cookies::SameSite;

#[derive(Clone, Debug, Default)]
pub struct Cookie {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) expires: Option<Duration>,
    pub(crate) max_age: Option<Duration>,
    pub(crate) domain: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) secure: Option<bool>,
    pub(crate) http_only: Option<bool>,
    pub(crate) same_site: Option<SameSite>
}

impl Cookie {
    pub fn new(k: impl Into<String>, v: impl Into<String>) -> Self {
        return Self {
            name: k.into(),
            value: v.into(),
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: None,
            http_only: None,
            same_site: None
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

    pub fn set_path(&mut self, value: &str) -> &mut Self {
        self.path = Some(value.to_string());

        return self;
    }

    pub fn set_domain(&mut self, value: &str) -> &mut Self {
        self.domain = Some(value.to_string());

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

    pub fn set_same_site(&mut self, value: SameSite) -> &mut Self {
        self.same_site = Some(value);

        return self;
    }

    pub fn parse(&self) -> String {
        let mut cookie = cookie::Cookie::new(self.name.to_string(), self.value.to_string());

        if let Some(expires) = self.expires {
            cookie.set_expires(OffsetDateTime::now_utc() + cookie::time::Duration::seconds(expires.as_secs() as i64));
        }

        if let Some(max_age) = self.max_age {
            cookie.set_max_age(cookie::time::Duration::new(max_age.as_secs() as i64, 0));
        }

        if let Some(path) = &self.path {
            cookie.set_path(path);
        }

        if let Some(domain) = &self.domain {
            cookie.set_domain(domain);
        }

        if let Some(secure) = self.secure {
            cookie.set_secure(secure);
        }

        if let Some(http_only) = self.http_only {
            cookie.set_http_only(http_only);
        }

        if let Some(same_site) = self.same_site {
            match same_site {
                SameSite::Strict => cookie.set_same_site(cookie::SameSite::Strict),
                SameSite::Lax    => cookie.set_same_site(cookie::SameSite::Lax),
                SameSite::None   => cookie.set_same_site(cookie::SameSite::None),
            }
        }

        return cookie.to_string();
    }
}