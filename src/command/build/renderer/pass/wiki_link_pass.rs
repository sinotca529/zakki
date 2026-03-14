use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// `[[パス]]` 形式のウィキリンクを Markdown のリンク構文に変換します。
/// タイトルマップにパスが存在する場合はタイトルをリンクテキストに使用し、
/// 存在しない場合はパスをそのままリンクテキストに使用します。
pub fn wiki_link_pass(
    markdown: &str,
    src_path: &Path,
    title_map: &HashMap<PathBuf, String>,
) -> String {
    let mut result = String::with_capacity(markdown.len());
    let mut rest = markdown;

    while let Some(open) = rest.find("[[") {
        result.push_str(&rest[..open]);
        rest = &rest[open + 2..];

        let Some(close) = rest.find("]]") else {
            // 閉じ括弧がない場合はそのまま出力する
            result.push_str("[[");
            continue;
        };

        let link_path = &rest[..close];
        rest = &rest[close + 2..];

        // リンク先のソースファイルの絶対パスを解決してタイトルを検索する
        let resolved = src_path
            .parent()
            .unwrap_or(Path::new(""))
            .join(link_path);
        let title = title_map
            .get(&resolved)
            .map(|s| s.as_str())
            .unwrap_or(link_path);

        // .md 拡張子を .html に変換する
        let html_path = if link_path.ends_with(".md") {
            format!("{}.html", &link_path[..link_path.len() - 3])
        } else {
            link_path.to_owned()
        };

        result.push_str(&format!("[{title}]({html_path})"));
    }

    result.push_str(rest);
    result
}
