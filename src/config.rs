use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

/// Configuration of an LLM.
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub api_type: Option<String>,
    pub api_version: Option<String>,
}

// #[derive(Deserialize, Debug, Clone)]
// pub enum ConfigList {
//     Many(Vec<Config>),
// }

// impl<'de, 'a: 'de + 'a> Deserialize<'de> for ConfigList<'a> {
//     fn deserialize<D'de>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         serde_json::Deserializer::from(deserializer)
//     }
// }

impl Config {
    /// Load the configuration from a file.
    pub fn from_file<P: AsRef<Path>>(
        file: P,
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        let config: Vec<Config> = serde_json::from_reader(reader)?;

        Ok(config)
    }
}

// impl ConfigList {
//     /// Load the configuration from a file.
//     pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, Box<dyn Error>>
// {         let file = File::open(file)?;
//         let reader = BufReader::new(file);
//         let mut deser = serde_json::Deserializer::from_reader(reader);
//         let config: Self = ConfigList::deserialize(&mut deser)?;
//         Ok(config)
//     }
// }
