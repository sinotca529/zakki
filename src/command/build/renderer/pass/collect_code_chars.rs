use pulldown_cmark::{Event, Tag, TagEnd};
use std::collections::BTreeSet;

/// コードブロックとインラインコードに出てくる文字を集めます。
///
/// ここで集めた文字だけをフォントのサブセットに残します。
///
/// 非公開の記事の本文も数えます。bloom filter は語の有無を問い合わせられるため
/// 本文を渡していませんが、こちらは全記事を混ぜた文字の集合しか残らず、
/// どの記事に出たかは分かりません。外したほうが漏れる量は減るものの、
/// そうすると非公開の記事だけ桁が揃わなくなります。
pub fn collect_code_chars(events: &[Event]) -> BTreeSet<char> {
    let mut chars = BTreeSet::new();
    let mut in_code_block = false;

    for e in events {
        match e {
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::End(TagEnd::CodeBlock) => in_code_block = false,
            Event::Code(t) => chars.extend(t.chars()),
            Event::Text(t) if in_code_block => chars.extend(t.chars()),
            _ => {}
        }
    }

    chars
}

#[cfg(test)]
mod test {
    use super::collect_code_chars;
    use pulldown_cmark::{Options, Parser};
    use std::collections::BTreeSet;

    fn chars_of(md: &str) -> BTreeSet<char> {
        let events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        collect_code_chars(&events)
    }

    fn set(s: &str) -> BTreeSet<char> {
        s.chars().collect()
    }

    #[test]
    fn collects_code_block_and_inline_code() {
        assert_eq!(chars_of("```\n┌─┐\n```\n"), set("┌─┐\n"));
        assert_eq!(chars_of("`あ`\n"), set("あ"));
    }

    /// 本文の文字まで入れると、日本語を書くだけでサブセットが膨らみます。
    #[test]
    fn ignores_body_text() {
        assert_eq!(chars_of("本文です\n"), BTreeSet::new());
        assert_eq!(chars_of("本文と `x` です\n"), set("x"));
    }

    /// コードブロックが閉じたあとの文字を拾わないことを確かめます。
    #[test]
    fn stops_at_the_end_of_a_code_block() {
        assert_eq!(chars_of("```\nab\n```\n\nあいう\n"), set("ab\n"));
    }
}
