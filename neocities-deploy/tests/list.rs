use assert_cmd::{cargo, pkg_name, prelude::*};
use indoc::indoc;
use mockito::Server;
use predicates::str::{contains, starts_with};
use std::{env, process::Command};

mod common;

#[test]
fn test_list() {
    let mut server = Server::new();

    let mock = server
        .mock("GET", "/list")
        .match_header("Accept", "application/json")
        .match_header("Accept-Charset", "utf-8")
        .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(indoc! {r#"{
            "result": "success",
            "files": [{
                "path": "index.html",
                "is_directory": false,
                "size": 1023,
                "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                "sha1_hash": "c8aac06f343c962a24a7eb111aad739ff48b7fb1"
            }, {
                "path": "not_found.html",
                "is_directory": false,
                "size": 271,
                "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                "sha1_hash": "cfdf0bda2557c322be78302da23c32fec72ffc0b"
            }, {
                "path": "images",
                "is_directory": true,
                "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000"
            }, {
                "path": "images/cat.png",
                "is_directory": false,
                "size": 16793,
                "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                "sha1_hash": "41fe08fc0dd44e79f799d03ece903e62be25dc7d"
            }]
        }"#})
        .create();

    unsafe {
        env::set_var("NEOCITIES_DEPLOY_API_URL", server.url());
    }

    let mut cmd = Command::new(cargo::cargo_bin!(pkg_name!()));
    let config = common::config_file("username:password", "/path/to/lorem");

    cmd.arg("-v").arg("list").arg("--config").arg(config.path());
    cmd.assert()
        .success()
        .stdout(starts_with("Listing site lorem.com"))
        .stdout(contains("         images/"))
        .stdout(contains("16.8 KB  images/cat.png"))
        .stdout(contains(" 1.0 KB  index.html"))
        .stdout(contains("  271 B  not_found.html"));

    mock.assert();
}
