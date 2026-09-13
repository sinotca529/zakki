use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use hmac::Hmac;
use rand::Rng;
use sha2::Sha256;

const ITERATIONS: u32 = 600_000;

pub fn encode_with_password(password: &str, data: &[u8]) -> Vec<u8> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    rand::rng().fill(&mut salt);
    rand::rng().fill(&mut nonce);

    let mut key = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, ITERATIONS, &mut key).unwrap();

    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(key));
    let ct = cipher.encrypt(&Nonce::from(nonce), data).unwrap();

    let mut out = Vec::with_capacity(32 + ct.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&ITERATIONS.to_be_bytes());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}
