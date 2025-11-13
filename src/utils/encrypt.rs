use std::io::Result;

use aes_gcm::{
    AeadCore,
    Aes256Gcm,
    Key,
    Nonce,
    aead::{Aead, KeyInit, OsRng}
};
use base64::{Engine, engine::general_purpose};

pub fn encrypt(key: &str, data: &str) -> Result<String> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let cipher_text = cipher.encrypt(&nonce, data.as_bytes()).unwrap();
    let mut combined = nonce.to_vec();

    combined.extend_from_slice(&cipher_text);

    let mut buffer = String::new();

    general_purpose::STANDARD.encode_string(combined, &mut buffer);

    return Ok(buffer);
}

pub fn decrypt(key: &str, hash: &str) -> Result<String> {
    let key = Key::<Aes256Gcm>::from_slice(key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let combined = general_purpose::STANDARD.decode(hash);

    if combined.is_err() {
        return Ok(String::new());
    }

    let session = combined.unwrap();
    let (nonce_bytes, ciphertext) = session.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();

    return Ok(String::from_utf8(plaintext).unwrap());
}