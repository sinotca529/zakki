use std::ops::BitXor;

use icu_segmenter::WordSegmenter;
use itertools::Itertools as _;

use super::fxhash::fxhash32_multi;

const SEED: u64 = 0x517cc1b727220a95;
fn fxhash64(word: &str) -> u64 {
    let mut v = 0u64;
    word.bytes().for_each(|c| {
        v = v.rotate_left(5).bitxor(c as u64).wrapping_mul(SEED);
    });
    v
}

/// FilterSize : byte length of the filter
/// NumHash: The number of hash functions
struct BloomFilter<const FilterSizeByte: usize, const NumHash: u8> {
    filter: [u8; FilterSizeByte],
}

impl<const FilterSizeByte: usize, const NumHash: u8> BloomFilter<FilterSizeByte, NumHash> {
    fn new() -> Self {
        Self {
            filter: [0; FilterSizeByte],
        }
    }

    fn insert_word(&mut self, word: &str) {
        let filter_size_bit = FilterSizeByte as u32 * 8;

        let hashes = fxhash32_multi(word)
            .map(|h| h % filter_size_bit)
            .take(NumHash as usize);

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
        let filter_size_bit = FilterSizeByte as u32 * 8;

        let hashes = fxhash32_multi(word)
            .map(|h| h % filter_size_bit)
            .take(NumHash as usize);

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
