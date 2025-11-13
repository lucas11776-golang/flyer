use std::io::Result;

use aes_gcm::{
    AeadCore,
    Aes256Gcm,
    Key,
    Nonce,
    aead::{Aead, KeyInit, OsRng}
};
use base64::{Engine, engine::general_purpose};

// TODO: refactor to later un-deprecated
pub fn encrypt(key_bytes: &str, plaintext: &str) -> Result<String> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).unwrap();
    let mut combined = nonce.to_vec();

    combined.extend_from_slice(&ciphertext);

    let mut buffer = String::new();

    general_purpose::STANDARD.encode_string(combined, &mut buffer);

    return Ok(buffer);
}

// TODO: refactor to later un-deprecated
pub fn decrypt(key_bytes: &str, encrypted_b64: &str) -> Result<String> {
    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let combined = general_purpose::STANDARD.decode(encrypted_b64).unwrap();
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();

    return Ok(String::from_utf8(plaintext).unwrap());
}