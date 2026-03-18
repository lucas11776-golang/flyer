use std::{thread::sleep, time::Duration};

use async_std::{fs::{read_dir, remove_file}, stream::StreamExt};

use crate::session::file::SessionData;

pub(crate) async fn cleanup(path: String, expires: Duration)  {
    tokio::spawn(async move {
        loop {
            match read_dir(path.clone()).await {
                Ok(mut entries) => {
                    while let Some(entry) = entries.next().await {
                        match entry {
                            Ok(file) => {
                                if let Ok(meta) = file.metadata().await {
                                    if let Ok(last) = meta.accessed() {
                                        if let Ok(passed) = last.elapsed() {
                                            if passed.as_secs() > expires.as_secs() {
                                                if let Err(_) = remove_file(file.path()).await { /*  TODO: some error */ }
                                            }
                                        }
                                    }
                                }
                            },
                            Err(_) => { /* TODO: some error */ },
                        }
                    }
                },
                Err(_) => { /* TODO: some error... */ },
            };

            sleep(Duration::from_secs((60 * 60) * 6)); // TODO: For testing move to top of loop
        }
    });
}

pub(crate) async fn load_session(path: String, session_id: String) -> SessionData {
    todo!()
}

pub(crate) async fn create_session(path: String, session_id: String) -> SessionData {
    todo!()
}

// Format: name|s:length:"value";
pub(crate) fn serialize(data: &SessionData) -> String {
    let mut buffer = String::new();

    for (key, val) in data {
        buffer.push_str(&format!("{}|{}", key, format!("s:{}:\"{}\";", val.len(), val)));
    }

    return buffer;
}

// Format: name|s:length:"value";
pub(crate) fn deserialize(raw: &str) -> SessionData {
    let mut data = SessionData::new();

    for part in raw.split(';').filter(|s| !s.is_empty()) {
        if let Some((key_part, val_part)) = part.split_once('|') {
            if let (Some(start), Some(end)) = (val_part.find('"'), val_part.rfind('"')) {
                data.insert(key_part.to_string(), String::from(&val_part[start + 1..end]));
            }
        }
    }

    return data;
}