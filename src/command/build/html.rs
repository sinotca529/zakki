use super::metadata::{Metadata, MetadataBuilder};
use crate::{
    command::build::pass::{
        adjust_link_to_md, convert_math, get_h1, highlight_code, read_yaml_header,
    },
    config::Config,
    path::{DstPath, SrcPath},
    read_asset,
    util::{encode_with_password, write_file},
};
use anyhow::Result;
use base64::prelude::*;
use dialoguer::Password;
use pulldown_cmark::{Options, Parser};
use std::sync::OnceLock;

pub struct Page {
    body: String,
    dst_path: DstPath,
    metadata: Metadata,
}

impl Page {
    pub fn from_md_file(src_path: &SrcPath) -> Result<Self> {
        assert!(src_path.is_md());
        let md_content = std::fs::read(src_path.get_ref())?;
        let md_content = std::str::from_utf8(&md_content)?;

        let mut metadata_builder = MetadataBuilder::default();
        let dst_path = src_path.to_dst_path();
        metadata_builder.path(dst_path.rel_path().to_owned());

        let mut events: Vec<_> = Parser::new_ext(md_content, Options::all()).collect();

        read_yaml_header(&events, &mut metadata_builder);
        get_h1(&events, &mut metadata_builder);

        let metadata = metadata_builder.build()?;

        adjust_link_to_md(&mut events);
        convert_math(&mut events);
        highlight_code(&mut events, metadata.highlights());

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, events.into_iter());

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

        format!(
            read_asset!("crypto.html"),
            encoded = encoded,
            path_to_root = self.dst_path.path_to_dst().to_str().unwrap()
        )
    }

    fn gen_html(&self, cfg: &Config) -> String {
        let html = format!(
            read_asset!("page.html"),
            tag_elems = self.tag_elems(self.metadata.tags()),
            data = self.metadata.date(),
            path_to_root = self.dst_path.path_to_dst().to_str().unwrap(),
            body = self.body,
            site_name = cfg.site_name(),
        );

        if self.metadata.crypto() {
            self.crypto_html(&html)
        } else {
            html
        }
    }

    pub fn metadata(self) -> Metadata {
        self.metadata
    }

    pub fn save(&self, cfg: &Config) -> Result<()> {
        let html = self.gen_html(cfg);
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
