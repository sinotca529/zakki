use base64::{prelude::BASE64_STANDARD, Engine as _};
use serde::Serialize;

use super::fxhash::fxhash32_multi;

fn serialize_bytes_in_base64<S: serde::Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    BASE64_STANDARD.encode(bytes).serialize(s)
}

#[derive(Default, Serialize)]
pub struct BloomFilter {
    /// フィルター
    #[serde(serialize_with = "serialize_bytes_in_base64")]
    filter: Vec<u8>,
    /// 使用するハッシュ関数の数
    num_hash: u8,
}

impl BloomFilter {
    /// 新しい Bloom フィルターを作成します。
    /// * `num_words` - フィルターに格納する単語の数
    /// * `fp` - 誤り許容率
    pub fn new(num_words: usize, fp: f64) -> Self {
        let num_words = num_words as f64;
        let num_bit = -num_words * fp.ln() / 2.0f64.ln().powi(2);
        let num_byte = (num_bit / 8.0).ceil() as u32;
        let num_hash = (num_bit * 2.0f64.ln() / num_words).ceil() as u8;
        Self {
            filter: vec![0; num_byte as usize],
            num_hash,
        }
    }

    pub fn insert_word(&mut self, word: &str) {
        let num_bit = self.filter.len() as u32 * 8;
        let hashes = fxhash32_multi(word)
            .map(|h| h % num_bit)
            .take(self.num_hash as usize);

        hashes.for_each(|hash| {
            self.filter[hash as usize / 8] |= 1 << (hash % 8);
        });
    }

    #[cfg(test)]
    pub fn contains(&self, word: &str) -> bool {
        let num_bit = (self.filter.len() as u32) * 8;
        let mut hashes = fxhash32_multi(word)
            .map(|h| h % num_bit)
            .take(self.num_hash as usize);

        hashes.all(|hash| (self.filter[hash as usize / 8] & (1 << (hash % 8))) != 0)
    }
}

#[cfg(test)]
mod test {
    use super::BloomFilter;

    #[test]
    fn test() {
        let mut filter = BloomFilter::new(25, 0.01);
        filter.insert_word("メロス");
        filter.insert_word("は");
        filter.insert_word("激怒");
        filter.insert_word("した");
        filter.insert_word("。");
        filter.insert_word("必ず");
        filter.insert_word("、");
        filter.insert_word("か");
        filter.insert_word("の");
        filter.insert_word("邪智");
        filter.insert_word("暴虐");
        filter.insert_word("の");
        filter.insert_word("王");
        filter.insert_word("を");
        filter.insert_word("除");
        filter.insert_word("かな");
        filter.insert_word("け");
        filter.insert_word("れ");
        filter.insert_word("ば");
        filter.insert_word("なら");
        filter.insert_word("ぬ");
        filter.insert_word("と");
        filter.insert_word("決意");
        filter.insert_word("した");
        filter.insert_word("。");

        assert!(filter.contains("メロス"));
        assert!(filter.contains("激怒"));
        assert!(!filter.contains("めろす"));
        assert!(!filter.contains("憤怒"));
    }
}
