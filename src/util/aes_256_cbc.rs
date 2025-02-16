use super::VecExt;
use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
use rand::Rng;
use sha2::{Digest, Sha256};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

pub fn encode_with_password(password: &str, data: &[u8]) -> Vec<u8> {
    let key = Sha256::digest(password);
    let key = key.as_slice().try_into().unwrap();
    encode_with_random_iv(key, data)
}

fn encode_with_random_iv(key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let mut iv = [0u8; 16];
    rand::rng().fill(&mut iv);
    encode(&iv, key, data)
}

fn encode(iv: &[u8; 16], key: &[u8; 32], data: &[u8]) -> Vec<u8> {
    let iv_vec: Vec<_> = iv.into();
    let cypher = Aes256CbcEnc::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data);
    iv_vec.extended(cypher)
}
