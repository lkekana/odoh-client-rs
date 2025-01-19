use anyhow::Result;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// Encapsulates various configuration parameters
#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub server: Server,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct Server {
    pub proxy: Option<String>,
    pub target: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: Server::default(),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            proxy: Some("https://ibksturm.synology.me/proxy".to_string()),
            target: "https://odoh.cloudflare-dns.com".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut fd = File::open(path)?;
        let mut toml = String::new();
        fd.read_to_string(&mut toml)?;
        Self::from_string(&toml)
    }

    pub fn from_string(toml: &str) -> Result<Config> {
        let c: Config = toml::from_str(toml)?;
        Ok(c)
    }
}
