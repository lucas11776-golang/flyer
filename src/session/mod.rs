use serde::{Deserialize, Serialize};

use crate::utils::Values;

pub mod cookie;
pub mod local;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Session {
    pub(crate) session: Values,
    pub(crate) flash: Values,
    pub(crate) errors: Values,
    pub(crate) old: Values,
}

impl Session {
    pub fn new() -> Self {
        return Self {
            session: Values::new(),
            flash: Values::new(),
            errors: Values::new(),
            old: Values::new(),
        };
    }
}

impl Session {
    pub fn session(&self) -> Values {
        return self
            .session
            .clone();
    }

    pub fn set(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self
            .session
            .insert(k.into(), v.into());
    }

    pub fn set_values(&mut self, values: Values) {
        for (k, v) in values {
            self.set(k, v);
        }
    }

    pub fn get(&self, k: impl Into<String>) -> String {
        return self
            .session
            .get(&k.into())
            .unwrap_or(&String::new())
            .into();
    }

    pub fn remove(&mut self, k: impl Into<String>) {
        self
            .session
            .remove(&k.into());
    }

    pub fn errors(&self) -> Values {
        return self.errors.clone();
    }

    pub fn set_error(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.errors.insert(k.into(), v.into());
    }

    pub fn set_errors(&mut self, errors: Values) {
        for (k, v) in errors {
            self.set_error(k, v);
        }
    }

    pub fn error(&self, k: impl Into<String>) -> String {
        return self
            .errors
            .get(&k.into())
            .unwrap_or(&String::new())
            .into();
    }

    pub fn remove_error(&mut self, k: impl Into<String>) {
        self
            .errors
            .remove(&k.into());
    }

    pub fn set_old(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.old.insert(k.into(), v.into());
    }

    pub fn set_olds(&mut self, values: Values) {
        for (k, v) in values {
            self.set_old(k, v);
        }
    }

    pub fn old(&self, k: impl Into<String>) -> String {
        return self
            .old
            .get(&k.into())
            .unwrap_or(&String::new())
            .into()
    }

    pub fn olds(&self) -> Values {
        return self
            .old
            .clone();
    }

    pub fn set_flash(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self
            .flash
            .insert(k.into(), v.into());
    }

    pub fn set_flashes(&mut self, values: Values) {
        for (k, v) in values {
            self.set_flash(k, v);
        }
    }

    pub fn flash(&self, k: impl Into<String>) -> String {
        return self
            .flash
            .get(&k.into())
            .unwrap_or(&String::new())
            .into();
    }

    pub fn flashes(&self) -> Values {
        return self
            .flash
            .clone();
    }
}