use crate::util::{self, BloomFilter};
use pulldown_cmark::{CodeBlockKind, Event, Tag};
use std::collections::HashSet;

/// 記事から検索用の bloom filter を作ります。
///
/// 数式は入れません。LaTeX を入れると `frac` や `sum` が索引に載り、
/// 数式を含む記事すべてに当たるためです。描画結果の `x2` も打たれない文字列です。
///
/// 画像の alt とコードブロックのキャプションを拾うため、
/// `convert_image` と `add_code_caption` より前に呼びます。
/// コードの中身も入れるため、`highlight_code` より前に呼びます。
pub fn make_bloom_filter(events: &[Event], title: &str, fp: f64) -> BloomFilter {
    let mut words: HashSet<String> = util::tokenize(title).into_iter().collect();

    for e in events {
        match e {
            Event::Text(t) | Event::Code(t) => words.extend(util::tokenize(t)),
            // コードブロックのキャプションは info string に入っている
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                if let Some((_, caption)) = info.split_once(':') {
                    words.extend(util::tokenize(caption));
                }
            }
            _ => {}
        }
    }

    let mut filter = BloomFilter::new(words.len(), fp);
    words.iter().for_each(|w| filter.insert_word(w));
    filter
}
