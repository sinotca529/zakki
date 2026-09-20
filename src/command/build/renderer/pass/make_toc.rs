use super::assign_header_id::SECTION_ID_PREFIX;
use super::{end_of, text_of};
use crate::command::build::renderer::html_component::{A, DETAILS, LI, OL, SUMMARY};
use pulldown_cmark::{Event, HeadingLevel, Tag};

/// 記事から目次 HTML を作ります。見出しがない場合は空文字を返します。
///
/// 本文が確定した後に呼びます。見出しの中身はテキストだけを拾うので、
/// 見出しに書いたコードや数式は目次には出ません。
pub fn make_toc(events: &[Event]) -> String {
    let mut items = Vec::new();

    for (i, e) in events.iter().enumerate() {
        let Event::Start(Tag::Heading { level, id, .. }) = e else {
            continue;
        };

        // h1 はページタイトル、h5 以下は目次に載せるには細かすぎる
        let depth = match level {
            HeadingLevel::H2 => 1,
            HeadingLevel::H3 => 2,
            HeadingLevel::H4 => 3,
            _ => continue,
        };

        let id = id
            .as_deref()
            .expect("見出しの id は assign_header_id が振る");

        // id は接頭辞と階層番号でできているので、番号の部分を目次の表示にも使う
        let number = id
            .strip_prefix(SECTION_ID_PREFIX)
            .expect("見出しの id には接頭辞が付く");
        let text = text_of(&events[(i + 1)..end_of(events, i)]);

        items.push(TocItem {
            depth,
            link: A
                .attr("href", format!("#{id}"))
                .text(format!("{number}. {text}")),
        });
    }

    if items.is_empty() {
        return String::new();
    }

    let summary = SUMMARY.text("目次");
    DETAILS
        .attr("class", "toc")
        .html(format!("{summary}{}", ol_html(&items)))
}

/// 目次に載せる見出し 1 つ分です。
struct TocItem {
    /// h2 を 1 とした深さ
    depth: usize,
    /// 見出しへのリンク
    link: String,
}

/// 見出しの並びから入れ子の `<ol>` を作ります。
///
/// 後ろから組み立てます。ある見出しに着く時点で、それより深い見出しは
/// すでに 1 つの `<ol>` にまとまっているためです。
fn ol_html(items: &[TocItem]) -> String {
    // stack[i] は深さ i + 1 の <li> の並び。後ろから積むので順序は逆
    let mut stack: Vec<Vec<String>> = Vec::new();

    for item in items.iter().rev() {
        while stack.len() < item.depth {
            stack.push(Vec::new());
        }

        // 自分より深い段があれば、子の <ol> として取り込む
        let children = match stack.len() > item.depth {
            true => list_html(stack.pop().unwrap()),
            false => String::new(),
        };

        stack[item.depth - 1].push(LI.html(format!("{}{children}", item.link)));
    }

    list_html(stack.pop().unwrap_or_default())
}

/// 逆順に積んだ `<li>` を順に戻して `<ol>` にします。
fn list_html(lis: Vec<String>) -> String {
    OL.html(lis.into_iter().rev().collect::<String>())
}

#[cfg(test)]
mod test {
    use super::super::assign_header_id;
    use super::make_toc;
    use pulldown_cmark::{Options, Parser};

    fn toc_of(md: &str) -> String {
        let mut events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        assign_header_id(&mut events);
        make_toc(&events)
    }

    #[test]
    fn nests_by_heading_level() {
        let toc = toc_of("## あ\n\n### い\n\n## う\n");
        assert_eq!(
            toc,
            concat!(
                r#"<details class="toc"><summary>目次</summary>"#,
                r##"<ol><li><a href="#s1">1. あ</a>"##,
                r##"<ol><li><a href="#s1.1">1.1. い</a></li></ol>"##,
                r##"</li><li><a href="#s2">2. う</a></li></ol>"##,
                "</details>",
            )
        );
    }

    /// 2 段戻るときに <ol> と <li> が釣り合うことを確かめます。
    #[test]
    fn closes_every_level_when_going_back_up() {
        let toc = toc_of("## あ\n\n### い\n\n#### う\n\n## え\n");
        assert_eq!(
            toc,
            concat!(
                r#"<details class="toc"><summary>目次</summary>"#,
                r##"<ol><li><a href="#s1">1. あ</a>"##,
                r##"<ol><li><a href="#s1.1">1.1. い</a>"##,
                r##"<ol><li><a href="#s1.1.1">1.1.1. う</a></li></ol>"##,
                "</li></ol>",
                r##"</li><li><a href="#s2">2. え</a></li></ol>"##,
                "</details>",
            )
        );
    }

    #[test]
    fn returns_empty_string_without_headings() {
        assert_eq!(toc_of("本文です。\n"), "");
    }

    /// 見出しに書いた修飾は未定義の扱いです。目次にはテキストだけが出ます。
    #[test]
    fn takes_only_text_from_headings() {
        let toc = toc_of("## `Vec<Event>` を*渡す*\n");
        assert!(
            toc.contains(r##"<a href="#s1">1. Vec&lt;Event> を渡す</a>"##),
            "{toc}"
        );
    }
}
