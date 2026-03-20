use crate::include_asset;
use crate::command::build::renderer::context::Metadata;
use itertools::Itertools as _;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn tag_link_html(tag: &str, index_url: &str) -> String {
    format!(r#"<a class="tag" href="{index_url}?tag={tag}">{tag}</a>"#)
}

fn adjust_path_origin(path: &str, path_to_root: &Path) -> String {
    if path.starts_with("http://") || path.starts_with("https://") || path.starts_with("/") {
        return path.to_string();
    }
    path_to_root.join(path).to_str().unwrap().to_string()
}

fn header(path_to_root: &Path, site_name: &str) -> String {
    format!(
        include_asset!("header.html"),
        path_to_root = path_to_root.to_str().unwrap(),
        site_name = site_name,
    )
}

fn head<'a>(
    path_to_root: &Path,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    title: &str,
) -> String {
    let css_list = css_list.map(|p| {
        format!(
            r#"<link rel="stylesheet" href="{}" />"#,
            adjust_path_origin(p, path_to_root)
        )
    });

    let js_list = js_list.map(|p| {
        format!(
            r#"<script type="text/javascript" src="{}" defer></script>"#,
            adjust_path_origin(p, path_to_root)
        )
    });

    format!(
        include_asset!("head.html"),
        path_to_root = path_to_root.to_str().unwrap(),
        css_list = css_list.collect::<String>(),
        js_list = js_list.collect::<String>(),
        title = title,
    )
}

fn tag_elems(tags: &[String], dst_root_dir: &Path) -> String {
    let index_url = dst_root_dir.join("index.html");
    let index_url = index_url.to_str().unwrap();
    let nsbp = "\u{00a0}";
    tags.iter()
        .map(|t| tag_link_html(t, index_url))
        .fold(String::new(), |acc, e| format!("{acc}{nsbp}{e}"))
}

pub fn cards_html(metas: &[Metadata]) -> String {
    metas
        .iter()
        .filter(|m| !m.is_sub)
        .map(|m| {
            let path = m.path.to_str().unwrap_or_default();
            let extra_class = if m.path.starts_with("private/") { " crypto" } else { "" };
            let tags_data = m.tags.join(",");
            let tag_links: String = m.tags.iter()
                .map(|t| tag_link_html(t, "index.html"))
                .collect();
            format!(
                include_asset!("card.html"),
                extra_class = extra_class,
                tags_data = tags_data,
                path = path,
                title = m.title,
                update = m.update,
                tag_links = tag_links,
            )
        })
        .collect()
}

pub fn all_tags_html(metas: &[Metadata]) -> String {
    let tag_set: BTreeSet<&String> = metas.iter().flat_map(|m| m.tags.iter()).collect();
    tag_set
        .iter()
        .map(|t| tag_link_html(t, "index.html"))
        .join(" ")
}

pub fn index_html<'a>(
    site_name: &str,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    footer: &str,
    cards: &str,
    tags: &str,
) -> String {
    let path_to_root = &PathBuf::from(".");
    let head = head(path_to_root, css_list, js_list, site_name);
    let header = header(path_to_root, site_name);
    format!(
        include_asset!("index.html"),
        head = head,
        header = header,
        footer = footer,
        cards = cards,
        tags = tags,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn page_html<'a>(
    path_to_root: &Path,
    site_name: &str,
    title: &str,
    create_date: &str,
    last_update_date: &str,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    tags: &[String],
    article: &str,
    footer: &str,
) -> String {
    let head = head(path_to_root, css_list, js_list, title);
    let header = header(path_to_root, site_name);
    let tag_elems = tag_elems(tags, path_to_root);
    format!(
        include_asset!("page.html"),
        head = head,
        header = header,
        title = title,
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        article = article,
        footer_text = footer,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn crypto_html<'a>(
    path_to_root: &Path,
    site_name: &str,
    title: &str,
    create_date: &str,
    last_update_date: &str,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    tags: &[String],
    encoded_body: &str,
    footer: &str,
) -> String {
    let head = head(path_to_root, css_list, js_list, title);
    let header = header(path_to_root, site_name);
    let tag_elems = tag_elems(tags, path_to_root);
    format!(
        include_asset!("crypto.html"),
        head = head,
        header = header,
        title = title,
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        encoded = encoded_body,
        footer_text = footer,
    )
}
