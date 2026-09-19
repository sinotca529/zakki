use crate::{command::build::renderer::url::Url, include_asset};

pub fn tag_link_html(tag: &str, url_to_root: &Url) -> String {
    let tag_t = escape_html_text(tag);
    let tag_q = escape_html_attr(&encode_query_value(tag));
    format!(r#"<a class="tag" href="{url_to_root}/index.html?tag={tag_q}">{tag_t}</a>"#)
}

pub fn header(url_to_root: &Url, site_name: &str) -> String {
    format!(
        include_asset!("header.html"),
        url_to_root = url_to_root,
        site_name = escape_html_text(site_name),
    )
}

pub fn footer(custom_footer: &Option<String>) -> String {
    custom_footer
        .as_ref()
        .map(|f| format!("<footer>{f}</footer>"))
        .unwrap_or_default()
}

pub fn head<'a>(
    url_to_root: &Url,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    title: &str,
) -> String {
    let css_list = css_list.map(|p| {
        format!(
            r#"<link rel="stylesheet" href="{}" />"#,
            adjust_path_origin(p, url_to_root)
        )
    });

    let js_list = js_list.map(|p| {
        format!(
            r#"<script type="text/javascript" src="{}" defer></script>"#,
            adjust_path_origin(p, url_to_root)
        )
    });

    format!(
        include_asset!("head.html"),
        url_to_root = url_to_root,
        css_list = css_list.collect::<String>(),
        js_list = js_list.collect::<String>(),
        title = escape_html_text(title),
    )
}

pub fn tag_elems(tags: &[String], url_to_root: &Url) -> String {
    tags.iter().map(|t| tag_link_html(t, url_to_root)).collect()
}

fn adjust_path_origin(path: &str, url_to_root: &Url) -> String {
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("/") {
        return path.to_string();
    }
    format!("{url_to_root}/{path}")
}

/// クエリ文字列の値として安全な形にします。
///
/// 非 ASCII はそのまま残します。ブラウザが URL を解決する時点で
/// パーセントエンコードするため動作に影響はなく、
/// HTML のソース上でタグ名が読めるほうが利点が大きいためです。
pub fn encode_query_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let meta_chars = r#"&#+%= "<>`"#;
    for c in s.chars() {
        match c {
            c if meta_chars.contains(c) || c.is_control() => {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).as_bytes() {
                    out.push_str(&format!("%{b:02X}"));
                }
            }
            c => out.push(c),
        }
    }
    out
}

/// HTML のテキスト内容として使えるようエスケープします。
pub fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;")
}

/// HTML の属性値として使えるようエスケープします。
pub fn escape_html_attr(attr: &str) -> String {
    attr.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}

/// クライアント側の実装と突き合わせるための表です。
#[cfg(test)]
mod test_vector {
    use crate::command::build::renderer::html_component::encode_query_value;

    #[derive(serde::Deserialize)]
    struct Case {
        r#in: String,
        out: String,
    }

    #[test]
    fn matches_table() {
        let src = crate::include_testdata!("encode_query_value.json");
        let cases: Vec<Case> = serde_json::from_str(src).unwrap();
        assert!(!cases.is_empty());
        for c in cases {
            assert_eq!(encode_query_value(&c.r#in), c.out, "入力: {:?}", c.r#in);
        }
    }
}

#[cfg(test)]
mod test {
    use super::adjust_path_origin;
    use crate::command::build::renderer::url::Url;
    use std::path::Path;

    #[test]
    fn authored_url_is_kept_as_written() {
        let root = Url::from_relative_path(Path::new("..")).unwrap();
        let f = |p| adjust_path_origin(p, &root);
        assert_eq!(f("katex/katex.min.css"), "../katex/katex.min.css");
        assert_eq!(f("a%20b.css?v=2"), "../a%20b.css?v=2");
        assert_eq!(f("https://example.com/x.css"), "https://example.com/x.css");
        assert_eq!(f("/assets/x.css"), "/assets/x.css");
    }
}
