use super::fxhash::fxhash32_multi;
use icu_segmenter::WordSegmenter;
use itertools::Itertools as _;
use std::ops::BitXor;

fn fxhash64(word: &str) -> u64 {
    const SEED: u64 = 0x517cc1b727220a95;
    let mut v = 0u64;
    word.bytes().for_each(|c| {
        v = v.rotate_left(5).bitxor(c as u64).wrapping_mul(SEED);
    });
    v
}

/// FILTER_SIZE : Byte length of the filter
/// NUM_HASH    : The number of hash functions
struct BloomFilter<const FILTER_SIZE: usize, const NUM_HASH: u8> {
    filter: [u8; FILTER_SIZE],
}

impl<const FILTER_SIZE: usize, const NUM_HASH: u8> BloomFilter<FILTER_SIZE, NUM_HASH> {
    fn new() -> Self {
        Self {
            filter: [0; FILTER_SIZE],
        }
    }

    fn insert_word(&mut self, word: &str) {
        let filter_size_bit = FILTER_SIZE as u32 * 8;

        let hashes = fxhash32_multi(word)
            .map(|h| h % filter_size_bit)
            .take(NUM_HASH as usize);

        hashes.for_each(|hash| {
            self.filter[hash as usize / 8] |= 1 << (hash % 8);
        });
    }

    /// Tokenize the given text and insert obtained words into the filter.
    pub fn insert_words(&mut self, text: &str) {
        let segmenter = WordSegmenter::new_auto();
        let words: Vec<_> = segmenter
            .segment_str(text)
            .iter_with_word_type()
            .tuple_windows()
            .filter(|(_, (_, segment_type))| segment_type.is_word_like())
            .map(|((i, _), (j, _))| &text[i..j])
            .collect();

        for word in &words {
            self.insert_word(word);
        }
    }

    #[cfg(test)]
    pub fn contains(&self, word: &str) -> bool {
        let filter_size_bit = FILTER_SIZE as u32 * 8;

        let hashes = fxhash32_multi(word)
            .map(|h| h % filter_size_bit)
            .take(NUM_HASH as usize);

        hashes
            .map(|hash| (self.filter[hash as usize / 8] & (1 << (hash % 8))) == 0)
            .any(|c| !c)
    }
}

#[cfg(test)]
mod test {
    use super::BloomFilter;

    #[test]
    fn test() {
        let s1 = "メロスは激怒した。必ず、かの邪智暴虐の王を除かなければならぬと決意した。";
        let mut filter = BloomFilter::<128, 3>::new();
        filter.insert_words(s1);
        assert!(filter.contains("メロス"));
        assert!(filter.contains("激怒"));
        assert!(!filter.contains("めろす"));
        assert!(!filter.contains("憤怒"));
    }
}
