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

use crate::params::{Config, Params, Site};
use anyhow::Result;
use inquire::validator::{ErrorMessage, Validation};
use neocities_client::ureq;
use neocities_client::Auth;
use std::path::{Path, PathBuf};
use url::Url;

/// Configure a site interactively.
pub fn config(params: &Params) -> Result<()> {
    eprintln!("Configuring sites interactively.");

    let (name, site) = login()?;
    let (name, site) = other_options(name, site)?;

    save_site(params.config_file(), name, site)?;

    Ok(())
}

/// Prompt the user for the login credentials, log in and build a preliminary [`Site`] object.
fn login() -> Result<(String, Site)> {
    let mut username = String::new();
    let mut proxy = String::new();
    loop {
        username = inquire::Text::new("Username:")
            .with_initial_value(&username)
            .with_help_message("Username for your NeoCities account")
            .with_validator(|s: &str| Ok(non_empty_validator(s)))
            .prompt()?;
        let password = inquire::Password::new("Password:")
            .with_help_message("Password for your NeoCities account")
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .without_confirmation()
            .prompt()?;
        proxy = inquire::Text::new("Proxy:")
            .with_initial_value(&proxy)
            .with_help_message("HTTP proxy to use, leave empty for none")
            .with_placeholder("https://proxy.example.com:8080")
            .with_validator(|s: &str| Ok(proxy_validator(s)))
            .with_formatter(&|s| (if s.is_empty() { "None" } else { s }).to_owned())
            .prompt()?;

        if let Ok((name, site)) = build_site(
            username.clone(),
            password,
            (!proxy.is_empty()).then(|| proxy.clone()),
        ) {
            break Ok((name, site));
        }
        eprintln!("Login failed! Try again, or press ^C to abort.");
    }
}

/// Prompt the user for the other configuration options.
fn other_options(name: String, site: Site) -> Result<(String, Site)> {
    let mut site = site;

    let name = inquire::Text::new("Name:")
        .with_initial_value(&name)
        .with_help_message("Name of the site")
        .with_validator(|s: &str| Ok(non_empty_validator(s)))
        .prompt()?;
    let path = directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_default();
    let path = inquire::Text::new("Path:")
        .with_initial_value(&format!("{}/", path.to_str().unwrap_or_default()))
        .with_help_message("Local path of the site")
        .with_validators(&[
            Box::new(&|s: &str| Ok(non_empty_validator(s))),
            Box::new(&|s: &str| Ok(path_validator(s))),
        ])
        .prompt()?;
    let free_account = inquire::Confirm::new("Free account?")
        .with_default(true)
        .with_help_message("Is this a free account?")
        .prompt()?;

    site.path = path;
    site.free_account = Some(free_account);

    Ok((name, site))
}

/// Save the site to the configuration file.
///
/// * If the site already exists, we'll ask the user if he wants to replace it.
/// * If the configuration file does not exist, it will be created.
fn save_site(config_file: impl Into<PathBuf>, name: String, site: Site) -> Result<()> {
    let config_file = config_file.into();
    let mut config = Config::load(&config_file).unwrap_or_else(|_| Default::default());
    if config.has_site(&name) {
        let replace = inquire::Confirm::new("Site already exists. Replace it?")
            .with_default(false)
            .prompt()?;
        if !replace {
            return Ok(());
        }
    }
    config.insert_site(name, site);
    config.save(&config_file)?;
    Ok(())
}

/// Build a [`Site`] object for the login function.
fn build_site(username: String, password: String, proxy: Option<String>) -> Result<(String, Site)> {
    let mut site = Site {
        auth: Auth::Credentials(username, password),
        path: "/".to_owned(),
        free_account: None,
        proxy: proxy.clone(),
    };
    let client = site.build_client()?;
    site.auth = Auth::ApiKey(client.key()?);
    let client = site.build_client()?;
    let name = client.info()?.sitename;
    Ok((name, site))
}

/// Validate a non-empty string.
fn non_empty_validator(s: &str) -> Validation {
    if s.is_empty() {
        Validation::Invalid(ErrorMessage::Custom(
            "Please enter a non-empty value.".to_owned(),
        ))
    } else {
        Validation::Valid
    }
}

/// Validate the proxy URL.
fn proxy_validator(s: &str) -> Validation {
    if s.is_empty() {
        Validation::Valid
    } else {
        if Url::parse(s).is_ok() && ureq::Proxy::new(s).is_ok() {
            return Validation::Valid;
        }
        Validation::Invalid(ErrorMessage::Custom("Invalid proxy URL".to_owned()))
    }
}

/// Validate the path to the local directory.
fn path_validator(s: &str) -> Validation {
    if !Path::new(s).is_dir() {
        Validation::Invalid(ErrorMessage::Custom(
            "Path does not exist or is not a directory".to_owned(),
        ))
    } else {
        Validation::Valid
    }
}
