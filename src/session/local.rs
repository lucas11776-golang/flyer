use std::{env, time::Duration, path::Path, fmt::Write};
use uuid::Uuid;
use tokio::io::AsyncReadExt;

use crate::cookies::SameSite;
use crate::utils::Values;
use crate::{
    hooks::Hook,
    request::Request,
    response::Response,
    routing::next::Next
};

use crate::session::Session; 

pub struct LocalSession {
    path: String,
    duration: Duration
}

impl Hook for LocalSession {
    async fn before(&self, mut req: Request, mut res: Response, next: Next) -> Response {
        let session_id = req.cookie("session-id");

        if !session_id.is_empty() {
            let file_path = Path::new(&self.path).join(session_id);

            if let Ok(mut file) = tokio::fs::File::open(&file_path).await {
                if let Ok(metadata) = file.metadata().await {
                    let mut is_expired = false;

                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > self.duration {
                                is_expired = true;
                            }
                        }
                    }

                    if is_expired {
                        drop(file);

                        let _ = tokio::fs::remove_file(&file_path).await;
                    } else {
                        let mut content = String::with_capacity(metadata.len() as usize);

                        if file.read_to_string(&mut content).await.is_ok() {
                            if let Some(session) = self.parse_session_file(&content) {
                                req.session = session; 
                                res.session.set_values(req.session.session());
                            }
                        }
                    }
                }
            }
        }

        next.handle(req, res)
    }

    async fn after(&self, req: Request, mut res: Response, next: Next) -> Response {
        let session_id = {
            let id = req.cookie("session-id");

            if req.cookie("session-id").is_empty() {
                format!("{}{}", Uuid::new_v4(), Uuid::new_v4())
            } else {
                id
            }
        };

        let original_serialized = self.serialize_session(&req.session);



        if let Some(new_serialized) = self.serialize_session(&res.session) {
            let should_write = match &original_serialized {
                Some(orig) => orig != &new_serialized,
                None => true,
            };

            if should_write {
                let file_path = Path::new(&self.path).join(&session_id);
                let _ = tokio::fs::write(file_path, new_serialized).await;
            }
        }


        let cookie = res
            .set_cookie("session-id", session_id)
            .set_expires(self.duration)
            .set_same_site(SameSite::Lax)
            .set_path("/");

        if let Ok(url) = url_domain_parse::Url::parse(&format!("http://{}", req.host.clone())) {
            cookie.set_domain(&url.base_host().unwrap_or(url.host().unwrap_or(String::new())));
        }


        let val = cookie.parse();

        next.handle(req, res.set_header("Set-Cookie", val))
    }
}

impl LocalSession {
    pub fn new(path: Option<impl Into<String>>, expires: Duration) -> Self {
        let path_str: String = path
            .map(|p| p.into())
            .unwrap_or_else(|| env::temp_dir().to_string_lossy().into());

        std::fs::create_dir_all(&path_str)
            .unwrap_or_else(|err| panic!("Failed to create session directory at '{}': {}", path_str, err));

     
        let my = Self {
            path: path_str,
            duration: expires,
        };

        // Hardcoded from now.
        my.spawn_cleanup_task(Duration::from_secs(60 * 30));

        return my;
    }

    pub fn spawn_cleanup_task(&self, interval: Duration) {
        let path = self.path.clone();
        let session_duration = self.duration;

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .spawn(async move {
                let mut interval_ticker = tokio::time::interval(interval);
                
                loop {
                    if let Ok(mut entries) = tokio::fs::read_dir(&path).await {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Ok(metadata) = entry.metadata().await {
                                if let Ok(modified) = metadata.modified() {
                                    if let Ok(elapsed) = modified.elapsed() {
                                        if elapsed > session_duration {
                                            let _ = tokio::fs::remove_file(entry.path()).await;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Wait for the next tick
                    interval_ticker.tick().await;
                }
            });
    }

    fn parse_session_file(&self, content: &str) -> Option<Session> {
        let mut session = Values::default();
        let mut flash = Values::default();
        let mut errors = Values::default();
        let mut old = Values::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some((name, rest)) = line.split_once("|s:") {
                if let Some((_len, mut value_str)) = rest.split_once(':') {
                    if value_str.starts_with('"') && value_str.ends_with('"') {
                        value_str = &value_str[1..value_str.len() - 1];
                    }

                    if let Ok(parsed_value) = serde_json::from_str::<Values>(value_str) {
                        match name {
                            "session" => session = parsed_value,
                            "flash" => flash = parsed_value,
                            "errors" => errors = parsed_value,
                            "old" => old = parsed_value,
                            _ => {}
                        }
                    }
                }
            }
        }

        Some(Session { session, flash, errors, old })
    }

    fn serialize_session(&self, session: &Session) -> Option<String> {
        let mut output = String::with_capacity(512);
        
        let fields = [
            ("session", &session.session),
            ("flash", &session.flash),
            ("errors", &session.errors),
            ("old", &session.old),
        ];

        for (i, (name, val)) in fields.iter().enumerate() {
            let json_str = serde_json::to_string(val).ok()?;
            if i > 0 {
                output.push('\n');
            }
            write!(
                &mut output, 
                "{}|s:{}:\"{}\"", 
                name, 
                json_str.len(), 
                json_str
            ).ok()?;
        }

        Some(output)
    }
}





// use rand::seq::SliceRandom;
// use rand::rngs::SmallRng;

// // Shuffles the string directly in memory. Zero heap allocations.
// fn shuffle_ascii_inplace(input: &mut str) {
//     let mut rng: SmallRng = rand::make_rng();
    
//     // Safety: Shuffling bytes of a strictly ASCII string is guaranteed 
//     // to remain valid UTF-8. 
//     unsafe {
//         let bytes = input.as_bytes_mut();
//         bytes.shuffle(&mut rng);
//     }
// }









































// /*
// Can you implement LocalSession `before` and `after`


// #SessionFormat: `name|s:length:"value"`

// name   = Is the name of the field from `Session`.
// length = Is the size of the json parse field from `Session`.
// value  = Is the json parsed field from `Session`.

// -------------------------------------------------------------------------------------------------------------------------------
// `before(&self, req: Request, res: Response, next: Next)`:
// -------------------------------------------------------------------------------------------------------------------------------

// This function is called when request is make the function should check if file exist in `path`` you will get the name of 
// the file in `req.cookie("session-id")` if the file exist you should read the file as `#SessionFormat`


// remember the struct Session exists in `req.session` and please check if the file has not touched in > `duration` delete the file
// that means the session has expired.


// -------------------------------------------------------------------------------------------------------------------------------
// `after(&self, req: Request, res: Response, next: Next)`:
// -------------------------------------------------------------------------------------------------------------------------------

// This function is called when the request cycle is done from middleware or controller here you should create file using
// `req.cookie("session-id")` as file and and parse `Session` using `#SessionFormat` also the res as `res.session`

// */

// ```rust
// use std::{env, time::Duration};

// use crate::{
//     hooks::Hook,
//     request::Request,
//     response::Response,
//     routing::next::Next
// };

// pub struct LocalSession {
//     path: String,
//     duration: Duration
// }

// impl LocalSession {
//     pub fn new(path: Option<impl Into<String>>, expires: Duration) -> Self {
//         return Self {
//             path: path
//                 .map(|p| p.into())
//                 .unwrap_or(env::temp_dir().to_string_lossy().into()),
//             duration: expires,
//         }
//     }
// }

// impl Hook for LocalSession {
//     async fn before(&self, req: Request, res: Response, next: Next) -> Response {
//         return next.handle(req, res);
//     }
    
//     async fn after(&self, req: Request, res: Response, next: Next) -> Response {
//         return next.handle(req, res);
//     }
// }

// // // This is session struct
// // #[derive(Clone, Debug, Default, Serialize, Deserialize)]
// // pub struct Session {
// //     pub(crate) session: Values,
// //     pub(crate) flash: Values,
// //     pub(crate) errors: Values,
// //     pub(crate) old: Values,
// // }
// ```