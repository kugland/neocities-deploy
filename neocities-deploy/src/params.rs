////////       This file is part of the source code for neocities-deploy, a command-       ////////
////////       line tool for deploying your Neocities site.                                ////////
////////                                                                                   ////////
////////                           Copyright © 2024  André Kugland                         ////////
////////                                                                                   ////////
////////       This program is free software: you can redistribute it and/or modify        ////////
////////       it under the terms of the GNU General Public License as published by        ////////
////////       the Free Software Foundation, either version 3 of the License, or           ////////
////////       (at your option) any later version.                                         ////////
////////                                                                                   ////////
////////       This program is distributed in the hope that it will be useful,             ////////
////////       but WITHOUT ANY WARRANTY; without even the implied warranty of              ////////
////////       MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                ////////
////////       GNU General Public License for more details.                                ////////
////////                                                                                   ////////
////////       You should have received a copy of the GNU General Public License           ////////
////////       along with this program. If not, see https://www.gnu.org/licenses/.         ////////

//! The params module unifies command-line arguments and configuration file handling.

use anyhow::{Result, anyhow};
use clap::{ArgAction::Count, Parser};
use directories::ProjectDirs;
use indexmap::IndexMap;
use neocities_client::{
    Auth, Client,
    ureq::{AgentBuilder, Proxy},
};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

#[derive(Debug, Parser)]
#[command(version, about, author, long_about = None)]
pub struct Params {
    /// Config file
    #[clap(short, long, global = true)]
    pub config: Option<PathBuf>,
    /// Select a site. (If not given, all sites are selected.)
    #[clap(short, long = "site", global = true)]
    pub sites: Vec<String>,
    /// Ignore errors and continue.
    #[clap(short, long, global = true)]
    pub ignore_errors: bool,
    /// More verbosity.
    #[clap(short, long, global = true, action = Count)]
    verbose: Option<u8>,
    /// Less verbosity.
    #[clap(short, long, global = true, action = Count)]
    quiet: Option<u8>,
    /// Subcommand
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Configure a site interactively.
    Config,
    /// Replace credentials with API keys in the config file.
    Key,
    /// List files on the site(s).
    List,
    /// Deploy local files to the site(s).
    Deploy,
}

impl Params {
    /// Get the configuration file path.
    pub fn config_file(&self) -> PathBuf {
        self.config
            .clone()
            .unwrap_or_else(Config::default_config_file)
    }

    /// Load configuration from configuration file specified in the command line.
    pub fn config(&self) -> Result<Config> {
        Config::load(self.config_file())
    }

    /// Get the verbosity level for this program.
    #[allow(dead_code)]
    pub fn verbosity(&self) -> log::LevelFilter {
        use log::LevelFilter::*;
        let numeric_level = 3_u8
            .saturating_add(self.verbose.unwrap_or(0))
            .saturating_sub(self.quiet.unwrap_or(0));
        match numeric_level {
            0 => Off,
            1 => Error,
            2 => Warn,
            3 => Info,
            4 => Debug,
            _ => Trace,
        }
    }

    /// Get the sites to work with, as specified in the command line or all the available sites
    /// if none is specified.
    pub fn sites(&self) -> Result<Vec<(String, Site)>> {
        let config = self.config().unwrap_or_default();

        let names: Vec<_> = if self.sites.is_empty() {
            config.sites.keys().collect()
        } else {
            self.sites.iter().collect()
        };

        names
            .into_iter()
            .map(|name| {
                let site = config
                    .sites
                    .get(name)
                    .ok_or_else(|| anyhow!("Site not found: {}", name))?;
                Ok((name.to_owned(), site.to_owned()))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
/// Main struct for the configuration file.
pub struct Config {
    /// The configured sites.
    #[serde(rename = "site")]
    pub sites: IndexMap<String, Site>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
/// Configuration for a site.
pub struct Site {
    /// Authentication method to use.
    pub auth: Auth,
    /// Whether the account is free or paid.
    pub free_account: Option<bool>,
    /// Path to the local directory.
    pub path: String,
    /// Proxy to use for HTTP requests.
    pub proxy: Option<String>,
}

impl Config {
    /// Load the configuration from a file.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        log::trace!("Loading configuration from {:?}", path);
        let contents = fs::read_to_string(&path)?;
        let config = toml::from_str(&contents)?;
        log::trace!("{:#?}", config);
        Ok(config)
    }

    /// Save the configuration to a file.
    ///
    /// When the file does not exist, it will be created; if parent directories do not exist, they
    /// will be created as well.
    pub fn save(&self, path: impl Into<PathBuf>) -> Result<()> {
        let path = path.into();
        log::debug!("Saving configuration to {:?}", path);
        log::trace!("{:#?}", self);
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            log::debug!("Creating parent directories for {:?}", path);
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, toml::to_string_pretty(self)?)?;
        log::info!("Configuration saved to {:?}", path);
        Ok(())
    }

    /// Whether a site is present in the configuration.
    pub fn has_site(&self, name: &str) -> bool {
        self.sites.contains_key(name)
    }

    /// Insert a site into the configuration.
    pub fn insert_site(&mut self, name: String, site: Site) {
        self.sites.insert(name, site);
    }

    /// Get the default configuration file path.
    pub fn default_config_file() -> PathBuf {
        let mut path = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
            .expect("Failed to get project directories")
            .config_dir()
            .to_path_buf();
        path.push("config.toml");
        path
    }
}

impl Site {
    /// Build a [`Client`] from the site configuration.
    pub fn build_client(&self) -> Result<Client> {
        let auth = self.auth.clone();
        let agent = {
            let mut builder = AgentBuilder::new();
            if let Some(proxy) = &self.proxy {
                builder = builder.proxy(Proxy::new(proxy)?)
            }
            builder.build()
        };
        let client = {
            let mut client_builder = Client::builder();
            if let Ok(mockito_address) = env::var("NEOCITIES_DEPLOY_API_URL") {
                client_builder.base_url(mockito_address);
            }
            client_builder.ureq_agent(agent).auth(auth).build()?
        };
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    const TOML: &str = r#"
        [site."lorem.com"]
        auth = "user:pass"
        path = "/path/to/lorem"
        proxy = "http://localhost:8080"

        [site."ipsum.com"]
        auth = "api_key"
        path = "/path/to/ipsum"
        proxy = "http://localhost:8081"
    "#;

    #[test]
    fn test_sites() {
        let config: Config = toml::from_str(TOML).unwrap();
        assert_equal(config.sites.keys(), vec!["lorem.com", "ipsum.com"]);
        let lorem = config.sites.get("lorem.com").unwrap();
        let ipsum = config.sites.get("ipsum.com").unwrap();
        assert_eq!(lorem.auth, Auth::from("user:pass"));
        assert_eq!(lorem.path, "/path/to/lorem");
        assert_eq!(lorem.proxy, Some("http://localhost:8080".to_string()));
        assert_eq!(ipsum.auth, Auth::from("api_key"));
        assert_eq!(ipsum.path, "/path/to/ipsum");
        assert_eq!(ipsum.proxy, Some("http://localhost:8081".to_string()));
    }

    #[test]
    fn test_save() {
        let config: Config = toml::from_str(TOML).unwrap();

        let tmpdir = tempfile::tempdir().unwrap();
        let path = tmpdir.path().join("subdirectory").join("config.toml");
        config.save(&path).unwrap();

        assert!(path.exists());
        let saved_config = Config::load(&path).unwrap();
        assert_eq!(config, saved_config);
    }
}
