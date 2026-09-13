use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use hmac::Hmac;
use rand::Rng;
use sha2::Sha256;

const ITERATIONS: u32 = 600_000;

// 出力の並び: salt(16) || 反復回数 (4, ビッグエンディアン) || nonce(12) || 暗号文 + タグ
// 反復回数変更時に js 側のコードを変更しなくて済むように反復回数を埋め込む
pub fn encode_with_password(password: &str, data: &[u8]) -> Vec<u8> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];

    let mut rng = rand::rng();
    rng.fill(&mut salt);
    rng.fill(&mut nonce);

    let mut key = [0u8; 32];
    pbkdf2::pbkdf2::<Hmac<Sha256>>(password.as_bytes(), &salt, ITERATIONS, &mut key)
        .expect("HMAC は任意長の鍵を受け付けるため、鍵長で失敗することはない");

    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from(key));
    let ct = cipher
        .encrypt(&Nonce::from(nonce), data)
        .expect("記事の大きさが AES-GCM の上限である約 64 GiB を超えることはない");

    [&salt[..], &ITERATIONS.to_be_bytes(), &nonce, &ct].concat()
}
