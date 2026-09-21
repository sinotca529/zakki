use crate::{command::build::renderer::url::Url, include_asset};
use std::marker::PhantomData;

pub fn tag_link_html(tag: &str, url_to_root: &Url) -> String {
    let href = format!("{url_to_root}/index.html?tag={}", encode_query_value(tag));
    A.attr("class", "tag").attr("href", href).text(tag)
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
        .map(|f| FOOTER.html(f))
        .unwrap_or_default()
}

pub fn head<'a>(
    url_to_root: &Url,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    title: &str,
) -> String {
    let css_list = css_list.map(|p| {
        LINK.attr("rel", "stylesheet")
            .attr("href", adjust_path_origin(p, url_to_root))
            .build()
    });

    let js_list = js_list.map(|p| {
        SCRIPT
            .attr("type", "text/javascript")
            .attr("src", adjust_path_origin(p, url_to_root))
            .flag("defer")
            .build()
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

/// HTML の要素を組み立てます。属性値と中身のエスケープは、この中でだけ行います。
///
/// 閉じタグを書くかどうかはタグで決まるので、印の型で分けています。
/// `IMG.text(..)` や `DIV.build()` が `<div/>` になることは起きません。
pub struct Element<K> {
    name: &'static str,
    attrs: String,
    kind: PhantomData<K>,
}

/// 閉じタグを書かない要素 (空要素) の印です。
pub struct Void;

/// 閉じタグを書く要素の印です。
pub struct Normal;

pub const A: Element<Normal> = Element::new("a");
pub const ASIDE: Element<Normal> = Element::new("aside");
pub const DETAILS: Element<Normal> = Element::new("details");
pub const DIV: Element<Normal> = Element::new("div");
pub const FIGCAPTION: Element<Normal> = Element::new("figcaption");
pub const FIGURE: Element<Normal> = Element::new("figure");
pub const FOOTER: Element<Normal> = Element::new("footer");
pub const IMG: Element<Void> = Element::new("img");
pub const LI: Element<Normal> = Element::new("li");
pub const LINK: Element<Void> = Element::new("link");
pub const OBJECT: Element<Normal> = Element::new("object");
pub const OL: Element<Normal> = Element::new("ol");
pub const P: Element<Normal> = Element::new("p");
pub const SCRIPT: Element<Normal> = Element::new("script");
pub const SPAN: Element<Normal> = Element::new("span");
pub const SUMMARY: Element<Normal> = Element::new("summary");

impl<K> Element<K> {
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            attrs: String::new(),
            kind: PhantomData,
        }
    }

    /// 属性を足します。値が空の場合は属性ごと書きません。
    /// 値のない属性は `flag` を使います。
    pub fn attr(mut self, name: &str, value: impl AsRef<str>) -> Self {
        let value = value.as_ref();
        if !value.is_empty() {
            self.attrs
                .push_str(&format!(r#" {name}="{}""#, escape_html_attr(value)));
        }
        self
    }

    /// 属性をまとめて足します。数が変わる場合に使います。
    pub fn attrs(self, attrs: &[(&str, &str)]) -> Self {
        attrs
            .iter()
            .fold(self, |tag, (name, value)| tag.attr(name, value))
    }

    /// 値を取らない属性 (真偽値属性) を足します。
    /// 書いてあれば真、なければ偽という扱いなので、値は空で構いません。
    pub fn flag(mut self, name: &str) -> Self {
        self.attrs.push_str(&format!(r#" {name}="""#));
        self
    }
}

impl Element<Void> {
    /// 要素を作ります。
    pub fn build(self) -> String {
        format!("<{}{}/>", self.name, self.attrs)
    }
}

impl Element<Normal> {
    /// 中身のない要素を作ります。
    pub fn build(self) -> String {
        self.html("")
    }

    /// 中身をテキストとして入れた要素を作ります。
    pub fn text(self, text: impl AsRef<str>) -> String {
        self.html(escape_html_text(text.as_ref()))
    }

    /// 開きタグと閉じタグの組を作ります。
    /// 中身に他のイベントが挟まり、1 回で組み立てられないときに使います。
    pub fn pair(self) -> (String, String) {
        (
            format!("<{}{}>", self.name, self.attrs),
            format!("</{}>", self.name),
        )
    }

    /// 中身を HTML としてそのまま入れた要素を作ります。
    pub fn html(self, inner_html: impl AsRef<str>) -> String {
        format!(
            "<{}{}>{}</{}>",
            self.name,
            self.attrs,
            inner_html.as_ref(),
            self.name
        )
    }
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
    use super::{IMG, P, SCRIPT, adjust_path_origin};
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

    #[test]
    fn escapes_attribute_values() {
        assert_eq!(
            IMG.attr("src", r#"x".png"#).build(),
            r#"<img src="x&quot;.png"/>"#
        );
    }

    #[test]
    fn escapes_text_content() {
        assert_eq!(P.text("a < b"), "<p>a &lt; b</p>");
    }

    /// 値が空の属性は、書いても区別が付きません。alt="" のように
    /// 空であることに意味がある場合は flag を使います。
    #[test]
    fn omits_attributes_with_an_empty_value() {
        assert_eq!(
            IMG.attr("src", "a.png").attr("alt", "").build(),
            r#"<img src="a.png"/>"#
        );
        assert_eq!(
            IMG.attr("src", "a.png").flag("alt").build(),
            r#"<img src="a.png" alt=""/>"#
        );
    }

    /// defer のような真偽値属性は、書いてあれば真という扱いです。
    #[test]
    fn writes_flags_without_a_value() {
        assert_eq!(
            SCRIPT.attr("src", "a.js").flag("defer").build(),
            r#"<script src="a.js" defer=""></script>"#
        );
    }
}
