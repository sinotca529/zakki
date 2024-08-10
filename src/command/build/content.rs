use crate::path::ContentPath;
use crate::util::PathExt;
use anyhow::{bail, Result};
use metadata::YamlHeader;
use std::{fs::File, io::Read, path::PathBuf};

mod metadata;

pub use metadata::{HighlightMacro, Metadata};

pub enum Content {
    Markdown { metadata: Metadata, content: String },
    Other { path: ContentPath },
}

impl Content {
    fn read_yaml_header(markdown: &str) -> Result<YamlHeader> {
        if let Some((_, s)) = markdown.split_once("---\n") {
            if let Some((header, _)) = s.split_once("\n---") {
                return serde_yaml::from_str(header).map_err(Into::into);
            }
        }
        bail!("Cannot find yaml header")
    }

    fn get_page_title(markdown: &str) -> Result<String> {
        if let Some((_, s)) = markdown.split_once("\n# ") {
            if let Some((title, _)) = s.split_once('\n') {
                return Ok(title.trim().to_string());
            }
        }
        bail!("Cannot find page title")
    }

    pub fn new(src_path: PathBuf) -> Result<Self> {
        let is_md = src_path.extension_is("md");
        let path = ContentPath::new(src_path)?;

        if !is_md {
            return Ok(Self::Other { path });
        }

        let markdown = {
            let mut file = File::open(path.src_path())?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            content
        };

        let yaml_header = Self::read_yaml_header(&markdown)?;
        let title = Self::get_page_title(&markdown)?;

        let metadata = Metadata {
            create_date: yaml_header.create_date,
            last_update_date: yaml_header.last_update_date,
            tags: yaml_header.tags,
            flags: yaml_header.flags,
            highlights: yaml_header.highlights,
            title,
            path,
        };

        Ok(Self::Markdown {
            metadata,
            content: markdown,
        })
    }
}
