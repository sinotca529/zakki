use crate::{
    command::build::code_font,
    command::build::renderer::{
        PageMetadata,
        html_component::{escape_html_attr, escape_html_text, footer, head, header, tag_link_html},
        url::Url,
    },
    config::ProjectConfig,
    include_asset, util,
};
use std::{collections::BTreeSet, path::Path};

pub fn render_index(
    cfg: &ProjectConfig,
    build_dir: &Path,
    metadatas: &[PageMetadata],
) -> anyhow::Result<()> {
    let cards = cards_html(metadatas);
    let tags = all_tags_html(metadatas);

    let css_list = cfg
        .code_font
        .iter()
        .map(|_| code_font::CSS_PATH)
        .chain(cfg.css_list.iter().map(String::as_str));

    let content = index_html(
        &cfg.site_name,
        css_list,
        cfg.js_list.iter().map(|p| p.as_str()),
        &footer(&cfg.footer),
        &cards,
        &tags,
    );

    let index_path = build_dir.join("index.html");
    util::write_file(index_path, content).map_err(Into::into)
}

fn cards_html(metas: &[PageMetadata]) -> String {
    let url_to_root = Url::from_relative_path(Path::new(".")).unwrap();

    metas
        .iter()
        .filter(|m| !m.is_sub)
        .map(|m| {
            let extra_class = [(m.is_private, " crypto"), (m.is_group, " group")]
                .into_iter()
                .filter_map(|(on, class)| on.then_some(class))
                .collect::<String>();

            let tag_links: String = m
                .tags
                .iter()
                .map(|t| tag_link_html(t, &url_to_root))
                .collect();

            format!(
                include_asset!("card.html"),
                extra_class = extra_class,
                path = escape_html_attr(&m.path.to_string()),
                title = escape_html_text(&m.title),
                update = m.update,
                tag_links = tag_links,
            )
        })
        .collect()
}

fn all_tags_html(metas: &[PageMetadata]) -> String {
    let tag_set: BTreeSet<&String> = metas.iter().flat_map(|m| m.tags.iter()).collect();
    let url_to_root = Url::from_relative_path(Path::new(".")).unwrap();
    tag_set
        .iter()
        .map(|t| tag_link_html(t, &url_to_root))
        .collect()
}

fn index_html<'a>(
    site_name: &str,
    css_list: impl Iterator<Item = &'a str>,
    js_list: impl Iterator<Item = &'a str>,
    footer: &str,
    cards: &str,
    tags: &str,
) -> String {
    let url_to_root = Url::from_relative_path(Path::new(".")).unwrap();
    let head = head(&url_to_root, css_list, js_list, site_name);
    let header = header(&url_to_root, site_name);
    format!(
        include_asset!("index.html"),
        head = head,
        header = header,
        footer = footer,
        cards = cards,
        tags = tags,
    )
}
