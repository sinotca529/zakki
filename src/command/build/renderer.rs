mod html_component;
mod index;
pub mod meta_renderer;
mod page_locs;
mod page_meta;
pub mod page_renderer;
mod pass;
mod url;

use crate::command::build::renderer::html_component::{escape_html_text, head, header, tag_elems};
use crate::command::build::renderer::url::Url;
use crate::include_asset;
use crate::util::Date;
use anyhow::Result;
use serde::Deserialize;

pub use index::render_index;
pub use page_locs::PageLocs;
pub use page_meta::PageMetadata;

const FRONT_MATTER_DELIMITER: &str = "---";

/// ファイルを BufReader で読み、YAML フロントマター部分だけ取り出して title を返します。
/// フロントマター終端の `---` に達した時点で読み込みを止めるため、大きなファイルでも効率的です。
pub fn extract_title_from_path(path: &std::path::Path) -> Result<Option<String>> {
    use std::io::{BufRead as _, BufReader};

    let file = std::fs::File::open(path)?;
    let mut lines = BufReader::new(file).lines();

    match lines.next() {
        Some(Ok(line)) if line == FRONT_MATTER_DELIMITER => {}
        _ => return Ok(None),
    }

    let mut yaml = String::new();
    for line in lines {
        let line = line?;
        if line == FRONT_MATTER_DELIMITER {
            break;
        }
        yaml.push_str(&line);
        yaml.push('\n');
    }

    if yaml.is_empty() {
        return Ok(None);
    }

    parse_title_from_yaml(&yaml)
}

fn parse_title_from_yaml(yaml: &str) -> Result<Option<String>> {
    #[derive(Deserialize)]
    struct TitleOnly {
        title: Option<String>,
    }

    let parsed: TitleOnly = serde_yaml::from_str(yaml)?;
    Ok(parsed.title)
}

#[allow(clippy::too_many_arguments)]
pub fn page_html<'a>(
    url_to_root: &Url,
    site_name: &str,
    title: &str,
    create_date: Date,
    last_update_date: Date,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    tags: &[String],
    article: &str,
    footer: &str,
) -> String {
    let head = head(url_to_root, css_list, js_list, title);
    let header = header(url_to_root, site_name);
    let tag_elems = tag_elems(tags, url_to_root);
    format!(
        include_asset!("page.html"),
        head = head,
        header = header,
        title = escape_html_text(title),
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        article = article,
        footer = footer,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn crypto_html<'a>(
    url_to_root: &Url,
    site_name: &str,
    title: &str,
    create_date: Date,
    last_update_date: Date,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    tags: &[String],
    encoded_body: &str,
    footer: &str,
) -> String {
    let head = head(url_to_root, css_list, js_list, title);
    let header = header(url_to_root, site_name);
    let tag_elems = tag_elems(tags, url_to_root);
    format!(
        include_asset!("crypto.html"),
        head = head,
        header = header,
        title = escape_html_text(title),
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        encoded = encoded_body,
        footer = footer,
    )
}
