use crate::{
    command::build::pass::{
        adjust_link_to_md, convert_math, get_h1, highlight_code, read_yaml_header,
    },
    path::{DstPath, SrcPath},
    util::write_file,
};
use anyhow::Result;
use derive_builder::Builder;
use indoc::formatdoc;
use pulldown_cmark::Options;
use serde::Serialize;
use std::path::PathBuf;
use yaml_rust2::{Yaml, YamlLoader};

#[derive(Builder, Serialize)]
pub struct PageMetadata {
    date: String,
    #[serde(rename = "path")]
    dst_rel_path: PathBuf,
    title: String,
    tags: Vec<String>,
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

    fn gen_html(&self) -> String {
        formatdoc! {r#"
            <!DOCTYPE html>
            <html lang="ja">
            <meta name="date" content="{data}">
            <head>
            <meta charset="UTF-8">
            <link rel="stylesheet" href="{path_to_css}">
            </head>
            <body>
            <span>{data}</span><br>
            {body}
            </body>
            </html>
        "#,
            data = self.metadata.date,
            path_to_css = self.dst_path.path_to_css().to_str().unwrap(),
            body = self.body,
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
