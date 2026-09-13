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

/// クライアント側の実装と突き合わせるための表です。
#[cfg(test)]
mod golden {
    #[derive(serde::Deserialize)]
    struct Case {
        r#in: String,
        hash: String,
    }

    #[test]
    fn matches_table() {
        let src = crate::include_testdata!("fxhash64.json");
        let cases: Vec<Case> = serde_json::from_str(src).unwrap();
        assert!(!cases.is_empty());
        for c in cases {
            let got = format!("{:016x}", super::fxhash64(&c.r#in));
            assert_eq!(got, c.hash, "入力: {:?}", c.r#in);
        }
    }
}
