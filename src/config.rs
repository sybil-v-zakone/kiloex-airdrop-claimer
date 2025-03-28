use crate::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bsc_rpc: String,
    pub sleep_range: [u64; 2],
    pub use_proxy: bool,
}

impl Config {
    const PATH: &str = "data/config.toml";

    async fn read_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let cfg_str = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&cfg_str)?)
    }

    pub async fn read_default() -> Self {
        Self::read_from_file(Self::PATH)
            .await
            .expect("default config to be valid")
    }
}
