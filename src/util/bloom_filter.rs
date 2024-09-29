use base64::{prelude::BASE64_STANDARD, Engine as _};

use super::fxhash::fxhash32_multi;

pub struct BloomFilter {
    filter: Vec<u8>,
    num_byte: u32,
    num_hash: u8,
}

impl BloomFilter {
    pub fn new(num_byte: u32, num_hash: u8) -> Self {
        Self {
            filter: vec![0; num_byte as usize],
            num_byte,
            num_hash,
        }
    }

    pub fn insert_word(&mut self, word: &str) {
        let num_bit = self.num_byte * 8;
        let hashes = fxhash32_multi(word)
            .map(|h| h % num_bit)
            .take(self.num_hash as usize);

        hashes.for_each(|hash| {
            self.filter[hash as usize / 8] |= 1 << (hash % 8);
        });
    }

    pub fn dump_as_base64(&self) -> String {
        BASE64_STANDARD.encode(&self.filter)
    }

    #[cfg(test)]
    pub fn contains(&self, word: &str) -> bool {
        let num_bit = (self.num_byte as u32) * 8;
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
        let mut filter = BloomFilter::new(128 * 8, 3);
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