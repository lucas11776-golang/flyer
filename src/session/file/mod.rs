use std::{collections::HashMap, env, time::Duration};

use anyhow::Result;
use async_std::task::block_on;
use tokio::runtime::Runtime;

use crate::{
    request::Request,
    response::Response,
    session::{SessionManager, file::utils::cleanup}
};

pub(crate) mod utils;

const SESSION_FILE_PREFIX: &str = "session_";
const SESSION_ID_NAME: &str = "session-id";

type SessionData = HashMap<String, String>;

pub struct FileSessionManager {
    path: String
}

impl FileSessionManager {
    pub fn new(path: Option<&str>) -> Self {
        let path = String::from(path.map(|p| p.trim_end_matches("/"))
            .unwrap_or(&String::from(env::temp_dir()
            .to_string_lossy())));
        let path_copy = path.clone();

        let _ = block_on(Runtime::new().unwrap().spawn(async move {
            cleanup(path_copy.clone(), Duration::from_secs((60 * 60) * 2)).await;
        }));
        
        return Self {
            path: path
        };
    }
}

impl SessionManager for FileSessionManager {
    fn setup<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        let session_id = req.cookies
            .cookies
            .get(SESSION_ID_NAME)
            .map(|id| String::from(id))
            .unwrap_or(uuid::Uuid::new_v4().to_string());

        println!("SESSION_ID -> {} {}", self.path, session_id);

        return Ok(())
    }

    fn teardown<'a>(&'a mut self, req: &'a mut Request, res: &'a mut Response) -> Result<()> {
        return Ok(())
    }
}