use std::{thread::sleep, time::Duration};

use anyhow::{Result};
use async_std::{fs::{File, read_dir, remove_file}, io::{ReadExt, WriteExt}, stream::StreamExt, task::block_on};
use tokio::runtime::Runtime;

use crate::{session::file::{FileStorage, SESSION_FILE_PREFIX}, utils::Values};

// TODO: Refactor All

pub(crate) fn cleanup(path: String, expires: Duration)  {
    let _ = block_on(Runtime::new().unwrap().spawn(async move {
        tokio::spawn(async move {
            loop {
                match read_dir(path.clone()).await {
                    Ok(mut entries) => {
                        while let Some(entry) = entries.next().await {
                            match entry {
                                Ok(file) => {
                                    let name = file.file_name();
                                    let filename = name.to_str().unwrap_or("").split("_").collect::<Vec<&str>>();

                                    if filename.len() == 3 && format!("{}_{}", filename[0], filename[1]).eq(SESSION_FILE_PREFIX) {
                                        if let Ok(meta) = file.metadata().await {
                                            if let Ok(last) = meta.accessed() {
                                                if let Ok(passed) = last.elapsed() {
                                                    if passed.as_secs() > expires.as_secs() {
                                                        // TODO: disable still testing can delete other program temp files...
                                                        if let Err(_) = remove_file(file.path()).await { /*  TODO: some error */ }
                                                    }
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
    }));
}

pub(crate) async fn load_session(path: &str, session_id: String) -> FileStorage {
    return match File::open(format!("{}/{}", path.trim_end_matches("/"), session_id)).await {
        Ok(mut file) => {
            let mut buffer = String::new();

            match file.read_to_string(&mut buffer).await {
                Ok(_) => deserialize(&buffer),
                Err(_) => FileStorage::default(),
            }
        },
        Err(_) => FileStorage::default(),
    };
}

pub(crate) async fn save_session(path: &str, session_id: String, storage: &FileStorage) -> Result<()> {
    return match File::create(format!("{}/{}", path.trim_end_matches("/"), session_id)).await {
        Ok(mut file) => {
            match file.write_all(serialize(storage).as_bytes()).await {
                Ok(_) => Ok(()),
                Err(err) => Err(err.into()),
            }
        },
        Err(err) => Err(err.into()),
    };
}

// Format: name|s:length:"value";
pub(crate) fn serialize(storage: &FileStorage) -> String {
    let format = |mut buffer: String, data: Values| -> String {
        for (key, val) in data {
            buffer.push_str(&format!("{}|{}", key, format!("s:{}:\"{}\";", val.len(), val)));
        }

        return buffer;
    };

    let data = {
        let mut hash = Values::new();

        hash.insert(String::from("values"), serde_json::to_string(&storage.values).unwrap());
        hash.insert(String::from("errors"), serde_json::to_string(&storage.errors).unwrap());
        hash.insert(String::from("old"), serde_json::to_string(&storage.old).unwrap());

        hash
    };

    return format(String::new(), data);
}

// Format: name|s:length:"value";
pub(crate) fn deserialize(raw: &str) -> FileStorage {
    let format = |raw: &str| -> Values {
        let mut hash = Values::new();

        for part in raw.split(';').filter(|s| !s.is_empty()) {
            if let Some((key_part, val_part)) = part.split_once('|') {
                if let (Some(start), Some(end)) = (val_part.find('"'), val_part.rfind('"')) {
                    hash.insert(key_part.to_string(), String::from(&val_part[start + 1..end]));
                }
            }
        }

        return hash;
    };

    let storage = format(raw);

    return FileStorage::new(
        serde_json::from_str::<Values>(&storage.get("values").unwrap_or(&String::from("{}"))).unwrap_or(Values::new()),
        serde_json::from_str::<Values>(&storage.get("errors").unwrap_or(&String::from("{}"))).unwrap_or(Values::new()),
        serde_json::from_str::<Values>(&storage.get("old").unwrap_or(&String::from("{}"))).unwrap_or(Values::new()),
    );
}