use std::ops::BitXor as _;

const SEED: u64 = 0x517cc1b727220a95;

pub fn fxhash64(s: &str) -> u64 {
    let mut v = 0u64;
    s.bytes().for_each(|c| {
        v = v.rotate_left(5).bitxor(c as u64).wrapping_mul(SEED);
    });
    v
}

pub fn fxhash32_multi(s: &str) -> impl Iterator<Item = u32> {
    let (hash1, hash2) = {
        let hash = fxhash64(s);
        (hash as u32, (hash >> 32) as u32)
    };
    (0..).map(move |i| hash1.wrapping_add(hash2.wrapping_mul(i)))
}
