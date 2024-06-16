use anyhow::bail;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    site_name: String,
}

impl Config {
    pub fn site_name(&self) -> &str {
        &self.site_name
    }

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
