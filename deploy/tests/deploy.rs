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
use mockito::{Matcher, Server};
use serial_test::serial;
use sha1::{Digest, Sha1};
use std::{env, fs};

mod common;

fn sha1_hex(s: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(s);
    format!("{:x}", hasher.finalize())
}

#[test]
#[serial]
fn test_deploy_uploads_and_deletes() {
    // ----- local tree -----
    let local = tempfile::tempdir().unwrap();
    let keep = b"keep_content\n";
    let changed_local = b"new content for changed\n";
    let new_file = b"brand new file\n";
    fs::write(local.path().join("keep.txt"), keep).unwrap();
    fs::write(local.path().join("changed.txt"), changed_local).unwrap();
    fs::write(local.path().join("new.txt"), new_file).unwrap();

    let keep_sha = sha1_hex(keep);
    let changed_remote_sha = sha1_hex(b"old content for changed\n");

    // ----- mock server -----
    let mut server = Server::new();

    let list_body = format!(
        r#"{{
            "result": "success",
            "files": [
                {{ "path": "changed.txt", "is_directory": false, "size": 25,
                   "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                   "sha1_hash": "{changed_remote_sha}" }},
                {{ "path": "gone.txt", "is_directory": false, "size": 5,
                   "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                   "sha1_hash": "00000000000000000000000000000000deadbeef" }},
                {{ "path": "keep.txt", "is_directory": false, "size": 13,
                   "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                   "sha1_hash": "{keep_sha}" }}
            ]
        }}"#
    );
    let list_mock = server
        .mock("GET", "/list")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(list_body)
        .create();

    let upload_changed = server
        .mock("POST", "/upload")
        .match_body(Matcher::Regex("new content for changed".to_owned()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"result":"success","message":"uploaded"}"#)
        .expect(1)
        .create();

    let upload_new = server
        .mock("POST", "/upload")
        .match_body(Matcher::Regex("brand new file".to_owned()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"result":"success","message":"uploaded"}"#)
        .expect(1)
        .create();

    let delete_gone = server
        .mock("POST", "/delete")
        .match_body(Matcher::UrlEncoded(
            "filenames[]".to_owned(),
            "gone.txt".to_owned(),
        ))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"result":"success","message":"deleted"}"#)
        .expect(1)
        .create();

    unsafe {
        env::set_var("NEOCITIES_DEPLOY_API_URL", server.url());
    }

    let config = common::config_file("apikeyvalue", local.path());

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-v")
        .arg("deploy")
        .arg("--config")
        .arg(config.path());
    cmd.assert().success();

    list_mock.assert();
    upload_changed.assert();
    upload_new.assert();
    delete_gone.assert();
}

#[test]
#[serial]
fn test_deploy_no_changes_no_writes() {
    let local = tempfile::tempdir().unwrap();
    let body = b"hello\n";
    fs::write(local.path().join("only.txt"), body).unwrap();

    let mut server = Server::new();
    let list_mock = server
        .mock("GET", "/list")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(
            r#"{{
                "result": "success",
                "files": [
                    {{ "path": "only.txt", "is_directory": false, "size": 6,
                       "updated_at": "Sat, 13 Feb 2016 03:04:00 -0000",
                       "sha1_hash": "{}" }}
                ]
            }}"#,
            sha1_hex(body)
        ))
        .create();
    // No upload/delete mocks: any call would 501 from mockito, failing the run.

    unsafe {
        env::set_var("NEOCITIES_DEPLOY_API_URL", server.url());
    }
    let config = common::config_file("apikeyvalue", local.path());

    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("deploy").arg("--config").arg(config.path());
    cmd.assert().success();
    list_mock.assert();
}

#[test]
#[serial]
fn test_deploy_no_sites() {
    // Empty config (file exists but no sites): should print and succeed.
    let f = tempfile::NamedTempFile::new().unwrap();
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("deploy").arg("--config").arg(f.path());
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("No sites to deploy"));
}
