use std::{env, time::Duration};

use anyhow::Result;
use async_std::task::block_on;
use serde::{Deserialize, Serialize};

use crate::{
    request::Request,
    response::Response,
    session::{Session, SessionManager, file::utils::{cleanup, load_session, save_session}}, utils::Values
};

pub(crate) mod utils;

// TODO: Refactor All

// session_{{session-id}}
const SESSION_FILE_PREFIX: &str = "flyer_session";
const SESSION_ID_NAME: &str = "session-id";

#[derive(Serialize, Deserialize, Default)]

#[derive(Debug)]
pub(crate) struct FileStorage {
    pub values: Values,
    pub errors: Values,
    pub old: Values,
}

impl FileStorage {
    pub fn new(values: Values, errors: Values, old: Values) -> Self {
        return Self {
            values: values,
            errors: errors,
            old: old
        };
    }
}

pub struct FileSessionManager {
    path: String
}

impl FileSessionManager {
    pub fn new(path: Option<&str>) -> Self {
        let manager = Self {
            path: String::from(path.map(|p| p.trim_end_matches("/"))
                .unwrap_or(&String::from(env::temp_dir().to_string_lossy())))
        };

        cleanup(manager.path.clone(),  Duration::from_secs((60 * 60) * 2));
        
        return manager;
    }
}

impl SessionManager for FileSessionManager {
    fn setup<'a>(&'a mut self, req: &'a mut Request, _res: &'a mut Response) -> Result<()> {
        let session_id = req.cookies
            .cookies
            .get(SESSION_ID_NAME)
            .map(|id| String::from(id))
            .unwrap_or(format!("{}_{}", SESSION_FILE_PREFIX, uuid::Uuid::new_v4().to_string().replace("-", "")));
        let storage = block_on(load_session(&self.path, session_id.clone()));

        req.session = Some(Box::new(SessionFile {
            session_id: session_id,
            values: storage.values,
            errors: storage.errors,
            old: storage.old,
            new_old: Values::new(),
            new_errors: Values::new(),
        }));

        return Ok(())
    }

    fn teardown<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        unsafe {
            let ptr = req.session.as_mut().unwrap() as *mut Box<dyn Session + 'static> as usize;
            let session = &mut **(ptr as *mut Box<SessionFile>);

            let saved = block_on(save_session(
                &self.path,
                session.session_id.clone(),
                &FileStorage::new(session.values.clone(), res.errors.clone(), res.old.clone())
            ));

            match saved {
                Ok(_) => { req.cookies.set("session-id", &session.session_id); },
                Err(_) => { /* TODO: file not save log. */ },
            }

            return Ok(())
        }
    }
}

pub struct SessionFile {
    pub(crate) session_id: String,
    pub(crate) values: Values,
    pub(crate) errors: Values,
    pub(crate) old: Values,
    pub(crate) new_old: Values,
    pub(crate) new_errors: Values,
}

impl Session for SessionFile {
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