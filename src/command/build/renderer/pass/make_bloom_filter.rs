use crate::util::{self, BloomFilter};
use pulldown_cmark::Event;
use std::collections::HashSet;

/// 記事から検索用の bloom filter を作ります。
///
/// 本文が確定した後に呼びます。各パスは、人が読む文字を `Text` として残し、
/// タグだけを生の HTML にします。ですからここでは `Text` と `Code` を拾えば済みます。
///
/// 数式は `convert_math` が KaTeX の HTML に変えた後なので入りません。
/// LaTeX を入れると `frac` や `sum` が索引に載り、数式を含む記事すべてに当たります。
/// 描画結果の `x2` も打たれない文字列です。
pub fn make_bloom_filter(events: &[Event], title: &str, fp: f64) -> BloomFilter {
    let mut words: HashSet<String> = util::tokenize(title).into_iter().collect();

    for e in events {
        match e {
            Event::Text(t) | Event::Code(t) => words.extend(util::tokenize(t)),
            _ => {}
        }
    }

    let mut filter = BloomFilter::new(words.len(), fp);
    words.iter().for_each(|w| filter.insert_word(w));
    filter
}
