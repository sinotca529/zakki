use super::PassAssets;
use anyhow::Context as _;
use katex::{KatexContext, OutputFormat, Settings};
use pulldown_cmark::Event;
use std::sync::LazyLock;

/// 数式を KaTeX でレンダリング済みの HTML に置き換えます。
pub fn convert_math(events: &mut [Event<'_>], pa: &mut PassAssets) -> anyhow::Result<()> {
    let settings = |display_mode| {
        Settings::builder()
            .output(OutputFormat::Html)
            .display_mode(display_mode)
            .build()
    };
    let display = settings(true);
    let inline = settings(false);

    // 記号表と関数表の構築は重いので、1 度だけ作って使い回す
    static CTX: LazyLock<KatexContext> = LazyLock::new(KatexContext::default);
    let mut math_used = false;

    for e in events.iter_mut() {
        let (latex, settings) = match e {
            Event::InlineMath(latex) => (latex, &inline),
            Event::DisplayMath(latex) => (latex, &display),
            _ => continue,
        };

        let html = katex::render_to_string(&CTX, latex, settings)
            .with_context(|| format!("数式の変換に失敗しました : {latex}"))?;

        *e = Event::InlineHtml(sort_attributes(&html).into());
        math_used = true;
    }

    if math_used {
        pa.css_paths.push("katex/katex.min.css".to_string());
    }

    Ok(())
}

/// タグの属性を名前順に並べ替えます。
///
/// katex-rs は属性を乱数で種を決めるハッシュ表 (`RapidHashMap`) に入れるため、
/// 書き出す順序が実行のたびに変わります。クレート側もテストで並べ替えてから
/// 比べる作りなので、順序は保証されません。描画は変わりませんが、同じ入力から
/// 同じサイトが出るように、ここで揃えます。
fn sort_attributes(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(lt) = rest.find('<') {
        out.push_str(&rest[..lt]);
        rest = &rest[lt..];

        // タグとして読めない形はそのまま通す
        match sorted_tag(rest) {
            Some((tag, len)) => {
                out.push_str(&tag);
                rest = &rest[len..];
            }
            None => {
                out.push('<');
                rest = &rest[1..];
            }
        }
    }

    out.push_str(rest);
    out
}

/// 先頭のタグを読み、属性を名前順に並べ替えた文字列と、読んだバイト数を返します。
/// 開きタグとして読めない場合は `None` を返します。
///
/// 属性の値は `"` で囲まれ、中に `>` を含まないものだけを扱います。
/// KaTeX の出力はこの形です。
fn sorted_tag(s: &str) -> Option<(String, usize)> {
    let b = s.as_bytes();
    let mut i = 1;

    while b.get(i).is_some_and(u8::is_ascii_alphanumeric) {
        i += 1;
    }
    if i == 1 {
        return None;
    }
    let name = &s[1..i];

    let mut attrs = Vec::new();
    let self_closing = loop {
        let space = i;
        while b.get(i) == Some(&b' ') {
            i += 1;
        }

        match b.get(i)? {
            b'>' => {
                i += 1;
                break false;
            }
            b'/' if b.get(i + 1) == Some(&b'>') => {
                i += 2;
                break true;
            }
            // 属性の前に区切りの空白がない
            _ if i == space => return None,
            _ => {}
        }

        let name_start = i;
        while b.get(i).is_some_and(|c| !b" =>".contains(c)) {
            i += 1;
        }
        let attr = &s[name_start..i];

        if b.get(i) != Some(&b'=') || b.get(i + 1) != Some(&b'"') {
            return None;
        }
        i += 2;

        let value_start = i;
        while b.get(i).is_some_and(|c| *c != b'"') {
            i += 1;
        }
        if b.get(i) != Some(&b'"') {
            return None;
        }
        let value = &s[value_start..i];
        i += 1;

        attrs.push((attr, value));
    };

    attrs.sort_unstable_by_key(|(name, _)| *name);

    let mut tag = String::with_capacity(i);
    tag.push('<');
    tag.push_str(name);
    for (name, value) in attrs {
        tag.push(' ');
        tag.push_str(name);
        tag.push_str("=\"");
        tag.push_str(value);
        tag.push('"');
    }
    tag.push_str(if self_closing { "/>" } else { ">" });

    Some((tag, i))
}

#[cfg(test)]
mod test {
    use super::sort_attributes;

    #[test]
    fn sorts_attributes_by_name() {
        assert_eq!(
            sort_attributes(r#"<svg width="1em" viewBox="0 0 1 1" height="2em">x</svg>"#),
            r#"<svg height="2em" viewBox="0 0 1 1" width="1em">x</svg>"#
        );
    }

    #[test]
    fn keeps_self_closing_and_bare_tags() {
        assert_eq!(
            sort_attributes(r#"<img src="a.png" alt="あ"/><br><span>本文</span>"#),
            r#"<img alt="あ" src="a.png"/><br><span>本文</span>"#
        );
    }

    /// 読めない形はそのまま通します。
    #[test]
    fn passes_through_unexpected_forms() {
        for html in ["1 < 2", "<span class=x>", "<span class=\"x>", "</span>"] {
            assert_eq!(sort_attributes(html), html, "入力: {html:?}");
        }
    }
}
