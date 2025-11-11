

use crate::{request::Request, response::Response};

pub(crate) struct Assets {
    path: String
}


impl Assets {
    pub fn new(path: String) -> Self {
        return Self {
            path: path
        }
    }

    pub async fn handle<'a>(&mut self, req: &'a mut Request, res: &'a mut Response) -> (&'a mut Request, &'a mut Response) {



        return (req, res)
    }
}