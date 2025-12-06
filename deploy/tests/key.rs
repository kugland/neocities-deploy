use assert_cmd::prelude::*;
use indoc::indoc;
use mockito::Server;
use predicates::str::{contains, starts_with};
use serial_test::serial;
use std::{collections::HashMap, env, process::Command};

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
