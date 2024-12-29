use crate::include_asset;
use std::path::{Path, PathBuf};

fn header(path_to_root: &Path, site_name: &str) -> String {
    format!(
        include_asset!("header.html"),
        path_to_root = path_to_root.to_str().unwrap(),
        site_name = site_name,
    )
}

fn head<'a>(
    path_to_root: &Path,
    css_list: impl Iterator<Item = &'a PathBuf>,
    js_list: impl Iterator<Item = &'a PathBuf>,
    title: &str,
) -> String {
    let css_list = css_list.map(|p| {
        format!(
            r#"<link rel="stylesheet" href="{}" />"#,
            path_to_root.join(p).to_str().unwrap()
        )
    });
    let js_list = js_list.map(|p| {
        format!(
            r#"<script defer type="text/javascript" src="{}"></script>"#,
            path_to_root.join(p).to_str().unwrap()
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
    let nsbp = "\u{00a0}";
    tags.iter()
        .map(|n| {
            let path = dst_root_dir.join("index.html");
            let path = path.to_str().unwrap();
            format!(r#"<a class="tag" href="{path}?tag={n}">{n}</a>"#)
        })
        .fold(String::new(), |acc, e| format!("{acc}{nsbp}{e}"))
}

pub fn index_html(path_to_root: &Path, site_name: &str, footer: &str) -> String {
    let header = header(path_to_root, site_name);
    format!(
        include_asset!("index.html"),
        header = header,
        site_name = site_name,
        footer = footer,
    )
}

pub fn page_html<'a>(
    path_to_root: &Path,
    site_name: &str,
    title: &str,
    create_date: &str,
    last_update_date: &str,
    css_list: impl Iterator<Item = &'a PathBuf>,
    js_list: impl Iterator<Item = &'a PathBuf>,
    tags: &[String],
    body: &str,
    footer: &str,
) -> String {
    let head = head(path_to_root, css_list, js_list, title);
    let header = header(path_to_root, site_name);
    let tag_elems = tag_elems(tags, path_to_root);

    format!(
        include_asset!("page.html"),
        head = head,
        header = header,
        tag_elems = tag_elems,
        create_date = create_date,
        last_update_date = last_update_date,
        body = body,
        footer_text = footer,
    )
}

pub fn crypto_html<'a>(
    path_to_root: &Path,
    site_name: &str,
    title: &str,
    create_date: &str,
    last_update_date: &str,
    css_list: impl Iterator<Item = &'a PathBuf>,
    js_list: impl Iterator<Item = &'a PathBuf>,
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
        create_date = create_date,
        last_update_date = last_update_date,
        tag_elems = tag_elems,
        header = header,
        encoded = encoded_body,
        footer_text = footer,
    )
}
