use super::end_of;
use crate::command::build::renderer::html_component::{A, LI, UL};
use crate::util::PathExt as _;
use pulldown_cmark::{CodeBlockKind, Event, Tag};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 子の一覧を差し込む目印。コードブロックの info string に書きます。
///
/// HTML コメントではなくコードブロックにしているのは、pulldown-cmark が
/// 構造化したイベントとして渡してくれるためです。文字列を探す処理が要りません。
/// `:` を含めないのは、`add_code_caption` が `:` の後ろをキャプションとして扱うためです。
const MARKER: &str = "zakki-children";

/// `zakki-children` のコードブロックを、同じディレクトリにある記事の一覧に置き換えます。
///
/// 対象は直下の記事だけです。入れ子のディレクトリは、その `index.md` が 1 件として並び、
/// さらに下の記事はそちらの一覧に出ます。
///
/// 子が 1 つもなければ、目印ごと取り除きます。
pub fn insert_child_list(
    events: &mut Vec<Event<'_>>,
    src_path: &Path,
    title_map: &HashMap<PathBuf, String>,
) {
    if !events.iter().any(is_marker) {
        return;
    }

    let html = child_list_html(src_path, title_map);
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;

    while i < events.len() {
        if !is_marker(&events[i]) {
            out.push(events[i].clone());
            i += 1;
            continue;
        }

        if !html.is_empty() {
            out.push(Event::Html(html.clone().into()));
        }
        i = end_of(events, i) + 1;
    }

    *events = out;
}

/// `src_path` と同じディレクトリにある記事を、ファイル名の辞書順に返します。
/// `index.md` 自身は含めません。
///
/// 日付ではなくファイル名で並べるのは、連載の順序を書き手が決められるようにするためです。
/// カテゴリをまとめるだけのページでも、同じ並べ方になります。
pub fn children_of<'a>(
    src_path: &Path,
    title_map: &'a HashMap<PathBuf, String>,
) -> Vec<(&'a Path, &'a str)> {
    let Some(dir) = src_path.parent() else {
        return Vec::new();
    };

    let mut children: Vec<_> = title_map
        .iter()
        .filter(|(path, _)| path.parent() == Some(dir) && path != &&src_path.normalized())
        .map(|(path, title)| (path.as_path(), title.as_str()))
        .collect();

    children.sort_by_key(|(path, _)| *path);
    children
}

fn is_marker(event: &Event) -> bool {
    matches!(
        event,
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) if info.trim() == MARKER
    )
}

fn child_list_html(src_path: &Path, title_map: &HashMap<PathBuf, String>) -> String {
    let items: String = children_of(src_path, title_map)
        .into_iter()
        .map(|(path, title)| {
            // 一覧を置くのは同じディレクトリの `index.md` なので、ファイル名だけで辿れます。
            let href = path.with_extension("html");
            let href = href.file_name().unwrap_or_default().to_string_lossy();

            LI.html(A.attr("href", href).text(title))
        })
        .collect();

    match items.is_empty() {
        true => String::new(),
        false => UL.attr("class", "child-list").html(items),
    }
}

#[cfg(test)]
mod test {
    use super::{children_of, insert_child_list};
    use pulldown_cmark::{Event, Options, Parser};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    fn title_map() -> HashMap<PathBuf, String> {
        [
            ("src/public/bar/index.md", "まとめ"),
            ("src/public/bar/020-second.md", "後編"),
            ("src/public/bar/010-first.md", "前編"),
            ("src/public/bar/baz/deep.md", "さらに下"),
            ("src/public/other.md", "よそ"),
        ]
        .into_iter()
        .map(|(path, title)| (PathBuf::from(path), title.to_owned()))
        .collect()
    }

    fn html_of(md: &str) -> String {
        let mut events: Vec<_> = Parser::new_ext(md, Options::empty()).collect();
        insert_child_list(
            &mut events,
            Path::new("src/public/bar/index.md"),
            &title_map(),
        );

        let mut buf = String::new();
        pulldown_cmark::html::push_html(&mut buf, events.into_iter());
        buf
    }

    /// 直下の記事だけを、ファイル名の辞書順に並べます。
    #[test]
    fn lists_children_in_the_directory() {
        let title_map = title_map();
        let children = children_of(Path::new("src/public/bar/index.md"), &title_map);
        let titles: Vec<_> = children.iter().map(|(_, title)| *title).collect();
        assert_eq!(titles, ["前編", "後編"]);
    }

    /// 目印を一覧に置き換えます。
    #[test]
    fn replaces_the_marker() {
        let html = html_of("## 目次\n\n```zakki-children\n```\n");
        assert!(html.contains(r#"<a href="010-first.html">前編</a>"#));
        assert!(html.contains(r#"<a href="020-second.html">後編</a>"#));
        assert!(!html.contains("zakki-children"));
    }

    /// 目印のないページは変わりません。
    #[test]
    fn leaves_other_code_blocks_alone() {
        let html = html_of("```rust\nfn main() {}\n```\n");
        assert!(html.contains("fn main()"));
        assert!(!html.contains("child-list"));
    }

    /// 子がいなければ、目印ごと取り除きます。
    #[test]
    fn removes_the_marker_when_there_is_no_child() {
        let mut events: Vec<_> =
            Parser::new_ext("```zakki-children\n```\n", Options::empty()).collect();
        insert_child_list(&mut events, Path::new("src/public/other.md"), &title_map());
        assert!(!events.iter().any(|e| matches!(e, Event::Html(_))));
    }
}
