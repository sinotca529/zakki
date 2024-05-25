use crate::{
    command::build::pass::{
        adjust_link_to_md, convert_math, get_h1, highlight_code, read_yaml_header,
    },
    path::{DstPath, SrcPath},
    util::{encode_with_password, write_file},
};
use anyhow::Result;
use base64::prelude::*;
use derive_builder::Builder;
use dialoguer::Password;
use indoc::formatdoc;
use pulldown_cmark::Options;
use serde::Serialize;
use std::{path::PathBuf, sync::OnceLock};
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Builder, Serialize)]
pub struct PageMetadata {
    date: String,
    #[serde(rename = "path")]
    dst_rel_path: PathBuf,
    title: String,
    tags: Vec<String>,
    crypto: bool,
}

impl PageMetadataBuilder {
    pub fn read_yaml(&mut self, yaml: &str) -> &mut Self {
        let yaml = YamlLoader::load_from_str(yaml)
            .ok()
            .and_then(|mut y| y.pop())
            .and_then(|y| y.into_hash());

        let Some(yaml) = yaml else {
            return self;
        };

        let date = yaml
            .get(&Yaml::String("date".to_owned()))
            .and_then(|date| date.as_str().map(str::to_string));

        if let Some(date) = date {
            self.date(date);
        }

        let tags: Vec<String> = yaml
            .get(&Yaml::String("tag".to_owned()))
            .and_then(|tags| tags.as_vec())
            .iter()
            .flat_map(|tags| tags.iter())
            .filter_map(|t| t.as_str().map(str::to_string))
            .collect();

        self.tags(tags);

        let crypto = yaml
            .get(&Yaml::String("crypto".to_owned()))
            .and_then(|date| date.as_bool())
            .unwrap_or(false);

        self.crypto(crypto);

        self
    }
}

pub struct Page {
    body: String,
    dst_path: DstPath,
    metadata: PageMetadata,
}

impl Page {
    pub fn from_md_file(src_path: &SrcPath) -> Result<Self> {
        assert!(src_path.is_md());
        let md_content = std::fs::read(src_path.get_ref())?;
        let md_content = std::str::from_utf8(&md_content)?;

        let dst_path = src_path.to_dst_path();

        let mut body = String::new();
        let mut title = String::new();

        let mut metadata_builder = PageMetadataBuilder::default();

        let parser = pulldown_cmark::Parser::new_ext(md_content, Options::all())
            .map(adjust_link_to_md)
            .map(convert_math)
            .map(highlight_code)
            .map(|e| get_h1(e, &mut title))
            .map(|e| read_yaml_header(e, &mut metadata_builder));

        pulldown_cmark::html::push_html(&mut body, parser);

        if title.is_empty() {
            title.push_str("(NoTitle)");
        }

        metadata_builder
            .title(title)
            .dst_rel_path(dst_path.rel_path().to_owned());

        let metadata = metadata_builder.build()?;

        Ok(Self {
            body,
            metadata,
            dst_path,
        })
    }

    fn tag_elem(&self, tag_name: &str) -> String {
        let path_to_tag = self.dst_path.path_to_dst().join("tag.html");
        let path_to_tag = path_to_tag.to_str().unwrap();
        format!(r#"<a class="tag" href="{path_to_tag}?tag={tag_name}">{tag_name}</a>"#)
    }

    fn tag_elems(&self, tag_elems: &[String]) -> String {
        let nsbp = "\u{00a0}";
        tag_elems
            .iter()
            .map(|n| self.tag_elem(n))
            .fold(String::new(), |acc, e| format!("{acc}{nsbp}{e}"))
    }

    fn crypto_html(&self, html: &str) -> String {
        let html = html.as_bytes();
        let cypher = encode_with_password(get_password(), html);
        let encoded = BASE64_STANDARD.encode(cypher);

        formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <head>
            <meta charset="UTF-8">
            <script type="text/javascript" src="{path_to_root}/script.js"></script>
            </head>
            <body data-page="crypto" data-cypher="{encoded}">
                <h1>This page is protected.</h1>
                <input autofocus type="password" id="keyInput" placeholder="Enter your secret key">
                <button onclick="decodeCypher()">Decode</button>
            </body>
            </html>
        "#,
            path_to_root = self.dst_path.path_to_dst().to_str().unwrap(),
        }
    }

    fn gen_html(&self) -> String {
        let html = formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <meta name="date" content="{data}">
            <head>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="{path_to_root}/style.css">
            </head>
            <body>
            <a href="{path_to_root}/index.html">Top Page</a><br>
            <span>{data}</span>
            {tag_elems}<br>
            <span></span>
            {body}
            </body>
            </html>
        "#,
            tag_elems = self.tag_elems(&self.metadata.tags),
            data = self.metadata.date,
            path_to_root = self.dst_path.path_to_dst().to_str().unwrap(),
            body = self.body,
        };

        if self.metadata.crypto {
            self.crypto_html(&html)
        } else {
            html
        }
    }

    pub fn metadata(self) -> PageMetadata {
        self.metadata
    }

    pub fn save(&self) -> Result<()> {
        let html = self.gen_html();
        write_file(self.dst_path.get_ref(), html).map_err(Into::into)
    }
}

fn get_password() -> &'static String {
    static PASSWORD: OnceLock<String> = OnceLock::new();
    PASSWORD.get_or_init(|| {
        Password::new()
            .with_prompt("Password for hidden pages")
            .interact()
            .unwrap()
    })
}
