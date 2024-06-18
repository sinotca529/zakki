use anyhow::bail;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FileConfig {
    site_name: String,
}

impl FileConfig {
    pub fn load() -> anyhow::Result<Self> {
        let pwd = std::env::current_dir()?;
        let cfg = std::fs::read_dir(pwd)?
            .filter_map(|f| f.ok())
            .map(|f| f.file_name())
            .find(|f| f == "config.toml");

        let Some(cfg) = cfg else {
            bail!("config.toml is not found.");
        };

        let cfg = std::fs::read(cfg)?;
        let cfg = std::str::from_utf8(&cfg)?;

        toml::from_str(cfg).map_err(Into::into)
    }
}

pub struct Config {
    site_name: String,
    render_draft: bool,
}

impl Config {
    pub fn new(file_config: FileConfig, render_draft: bool) -> Self {
        Self {
            site_name: file_config.site_name,
            render_draft,
        }
    }

    pub fn render_draft(&self) -> bool {
        self.render_draft
    }

    pub fn site_name(&self) -> &str {
        &self.site_name
    }
}
