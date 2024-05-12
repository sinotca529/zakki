use std::{io, path::Path};

pub fn write_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let path = path.as_ref();
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, contents)
}

pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    let to = to.as_ref();
    std::fs::create_dir_all(to.parent().unwrap())?;
    std::fs::copy(from, to)
}

pub trait VecExt {
    fn extended(self, other: Self) -> Self;
}

impl<T> VecExt for Vec<T> {
    fn extended(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

mod aes_256_cbc {
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
        rand::thread_rng().fill(&mut iv);
        encode(&iv, key, data)
    }

    fn encode(iv: &[u8; 16], key: &[u8; 32], data: &[u8]) -> Vec<u8> {
        let iv_vec: Vec<_> = iv.into();
        let cypher = Aes256CbcEnc::new(key.into(), iv.into()).encrypt_padded_vec_mut::<Pkcs7>(data);

        iv_vec.extended(cypher)
    }
}

pub use aes_256_cbc::*;
