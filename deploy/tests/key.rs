////////       This file is part of the source code for neocities-deploy, a command-       ////////
////////       line tool for deploying your Neocities site.                                ////////
////////                                                                                   ////////
////////                       Copyright © 2024–2026   André Kugland                       ////////
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

use assert_cmd::Command;
use indoc::indoc;
use mockito::Server;
use predicates::str::{contains, starts_with};
use serial_test::serial;
use std::{collections::HashMap, env};

mod common;

#[test]
#[serial]
fn test_key() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/key")
        .match_header("Accept", "application/json")
        .match_header("Accept-Charset", "utf-8")
        .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{ "result": "success", "api_key": "c6275ca833ac06c83926ccb00dff4c82" }"#)
        .create();

    env::set_var("NEOCITIES_DEPLOY_API_URL", server.url());

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let config = common::config_file("username:password", "/path/to/lorem");

    cmd.arg("-v").arg("key").arg("--config").arg(config.path());
    cmd.assert()
        .success()
        .stdout(starts_with("Getting API key for site lorem.com\n"))
        .stderr(contains("Saving configuration to "))
        .stderr(contains("Configuration saved to "));

    mock.assert();
    drop(server);

    let my_toml: HashMap<String, HashMap<String, HashMap<String, String>>> =
        toml::from_str(&std::fs::read_to_string(config.path()).unwrap()).unwrap();

    assert_eq!(
        my_toml["site"]["lorem.com"]["auth"],
        "c6275ca833ac06c83926ccb00dff4c82"
    );
}

#[test]
#[serial]
fn test_key_error() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/key")
        .match_header("Accept", "application/json")
        .match_header("Accept-Charset", "utf-8")
        .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(indoc! {r#"{
            "result": "error",
            "error_type": "invalid_auth",
            "message": "invalid credentials - please check your username and password (or your api key)"
        }"#})
        .create();

    env::set_var("NEOCITIES_DEPLOY_API_URL", server.url());

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let config = common::config_file("username:password", "/path/to/lorem");

    cmd.arg("-v").arg("key").arg("--config").arg(config.path());
    cmd.assert()
        .failure()
        .stdout(starts_with("Getting API key for site lorem.com\n"))
        .stderr(contains(concat!(
            "Error: API error: invalid credentials - please check your ",
            "username and password (or your api key) (invalid_auth)\n"
        )));

    mock.assert();
    drop(server);

    let my_toml: HashMap<String, HashMap<String, HashMap<String, String>>> =
        toml::from_str(&std::fs::read_to_string(config.path()).unwrap()).unwrap();

    assert_eq!(my_toml["site"]["lorem.com"]["auth"], "username:password");
}
