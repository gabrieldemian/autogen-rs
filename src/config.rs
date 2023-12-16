//! Configuration that is used by agents.
use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

/// Configuration of an LLM.
///
/// # Important
/// The configuration must be in JSON format.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub api_type: Option<String>,
    pub api_version: Option<String>,
}

impl Config {
    /// Load the configuration from a json file.
    ///
    /// # Important
    /// The configuration must be in JSON format.
    pub fn from_file<P: AsRef<Path>>(
        file: P,
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        let config: Vec<Config> = serde_json::from_reader(reader)?;

        Ok(config)
    }
}
