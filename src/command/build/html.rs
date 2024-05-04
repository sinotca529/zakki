use crate::{
    command::build::pass::{adjust_link_to_md, convert_math, highlight_code},
    path::{DstPath, SrcPath},
    util::{write_file, DateFormat},
};
use anyhow::Result;
use indoc::formatdoc;
use pulldown_cmark::Options;
use std::time::SystemTime;

struct PageMetadata {
    last_update: SystemTime,
    dst_path: DstPath,
}

pub struct Page {
    body: String,
    metadata: PageMetadata,
}

impl Page {
    pub fn from_md_file(src_path: &SrcPath) -> Result<Self> {
        assert!(src_path.is_md());
        let md_content = std::fs::read(src_path.get_ref())?;
        let md_content = std::str::from_utf8(&md_content)?;

        let dst_path = src_path.to_dst_path();

        let mut body = String::new();

        let parser = pulldown_cmark::Parser::new_ext(md_content, Options::all())
            .map(adjust_link_to_md)
            .map(convert_math)
            .map(highlight_code);

        pulldown_cmark::html::push_html(&mut body, parser);

        let last_update = std::fs::metadata(src_path.get_ref()).and_then(|m| m.modified())?;
        let metadata = PageMetadata {
            last_update,
            dst_path,
        };
        Ok(Self { body, metadata })
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
            <span>Last update: {data}</span><br>
            {body}
            </body>
            </html>
        "#,
            data = self.metadata.last_update.yyyy_mm_dd_utc(),
            path_to_css = self.metadata.dst_path.path_to_css().to_str().unwrap(),
            body = self.body,
        }
    }

    pub fn save(&self) -> Result<()> {
        let html = self.gen_html();
        write_file(self.metadata.dst_path.get_ref(), html).map_err(Into::into)
    }
}
