////////       This file is part of the source code for neocities-client, a Rust           ////////
////////       library for interacting with the https://neocities.org/ API.                ////////
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

//! Client for the Neocities API.

use crate::response::{parse_response, Info, ListEntry};
use crate::{Auth, Error, Result};
use derive_builder::Builder;
use form_data_builder::FormData;
use std::{ffi::OsStr, io::Cursor};
use tap::prelude::*;
use typed_path::Utf8UnixPath;
use ureq::{Agent, RequestBuilder};
use ureq::typestate::{WithBody, WithoutBody};

/// Default base URL for the Neocities API.
const DEFAULT_BASE_URL: &str = "https://neocities.org/api";

/// Default user agent to use for the requests.
const DEFAULT_USER_AGENT: &str = concat!("neocities_client/", env!("CARGO_PKG_VERSION"));

/// List of file extensions allowed for free accounts.
const ALLOWED_EXTS_FOR_FREE_ACCOUNTS: &[&str] = &[
    "apng",
    "asc",
    "atom",
    "avif",
    "bin",
    "css",
    "csv",
    "dae",
    "eot",
    "epub",
    "geojson",
    "gif",
    "gltf",
    "gpg",
    "htm",
    "html",
    "ico",
    "jpeg",
    "jpg",
    "js",
    "json",
    "key",
    "kml",
    "knowl",
    "less",
    "manifest",
    "map",
    "markdown",
    "md",
    "mf",
    "mid",
    "midi",
    "mtl",
    "obj",
    "opml",
    "osdx",
    "otf",
    "pdf",
    "pgp",
    "pls",
    "png",
    "rdf",
    "resolveHandle",
    "rss",
    "sass",
    "scss",
    "svg",
    "text",
    "toml",
    "tsv",
    "ttf",
    "txt",
    "webapp",
    "webmanifest",
    "webp",
    "woff",
    "woff2",
    "xcf",
    "xml",
    "yaml",
    "yml",
];

/// Client for the Neocities API.
///
/// This struct is used to make requests to the Neocities API. It can be built using the
/// [`Client::builder()`](#method.builder) method, which returns a
/// [`ClientBuilder`](struct.ClientBuilder.html) struct.
///
/// ```
/// # use neocities_client::{Auth, Client};
/// let client = Client::builder()
///    .auth(Auth::from("username:password"))
///    .build()
///    .unwrap();
/// ```
#[derive(Debug, Builder)]
pub struct Client {
    /// Instance of [`ureq::Agent`] to use for the requests.
    ///
    /// Override this if you want to customize the [`Agent`](ureq::Agent), for example, to use a
    /// proxy, to set a timeout, to add middlewares, *&c*.
    #[builder(default = "Agent::config_builder().http_status_as_error(false).build().into()")]
    ureq_agent: Agent,
    /// Base URL for the Neocities API.
    ///
    /// Defaults to `https://neocities.org/api`.
    ///
    /// This is overridable for testing purposes.
    #[builder(default = "DEFAULT_BASE_URL.to_owned()")]
    base_url: String,
    /// User agent to use for the requests
    ///
    /// Defaults to `neocities_client/x.y.z`
    #[builder(default = "DEFAULT_USER_AGENT.to_owned()")]
    user_agent: String,
    /// Authorization that will be used for the requests.
    auth: Auth,
}

/// Client for the Neocities API.
#[allow(clippy::result_large_err)]
impl Client {
    /// Return a new [`ClientBuilder`] struct.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Delete one or more files from the website.
    pub fn delete(&self, paths: &[&str]) -> Result<()> {
        #[cfg(debug_assertions)]
        log::trace!("Deleting files {:?}", paths);
        let form = paths
            .iter()
            .map(|path| ("filenames[]", *path))
            .collect::<Vec<_>>();
        self.make_post_request("delete")
            .send_form(form)
            .map_err(Error::from)
            .and_then(|res| parse_response::<String>("message", res))
            .tap_ok_dbg(|msg| log::trace!("{}", msg))
            .tap_err(|e| log::debug!("{}", e))
            .and(Ok(()))
    }

    /// Get the website info.
    pub fn info(&self) -> Result<Info> {
        #[cfg(debug_assertions)]
        log::trace!("Getting website info");
        self.make_get_request("info")
            .call()
            .map_err(Error::from)
            .and_then(|res| parse_response::<Info>("info", res))
            .tap_ok_dbg(|info| log::trace!("{:?}", info))
            .tap_err(|e| log::debug!("{}", e))
    }

    /// Get an API key for the website.
    pub fn key(&self) -> Result<String> {
        #[cfg(debug_assertions)]
        log::trace!("Getting API key");
        self.make_get_request("key")
            .call()
            .map_err(Error::from)
            .and_then(|res| parse_response::<String>("api_key", res))
            .tap_ok_dbg(|_| log::trace!("Got an API key: <redacted>"))
            .tap_err(|e| log::debug!("{}", e))
    }

    /// List the files on the website.
    pub fn list(&self) -> Result<Vec<ListEntry>> {
        #[cfg(debug_assertions)]
        log::trace!("Listing files");
        self.make_get_request("list")
            .call()
            .map_err(Error::from)
            .and_then(|res| parse_response::<Vec<ListEntry>>("files", res))
            .tap_ok_dbg(|list| log::trace!("{:?}", list))
            .tap_err(|e| log::debug!("{}", e))
    }

    /// Upload one or more files to the website.
    ///
    /// This method receives a list of tuples, each containing the path of the file and the
    /// contents of the file.
    ///
    /// ```no_run
    /// # use neocities_client::{Auth, Client, Result};
    /// # fn main() -> Result<()> {
    /// # let client = Client::builder().auth(Auth::from("faketoken")).build().unwrap();
    /// client.upload(&[
    ///     ("/1st_file.txt", b"Contents of the first file"),
    ///     ("/2nd_file.txt", b"Contents of the second file"),
    /// ])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn upload(&self, files: &[(&str, &[u8])]) -> Result<()> {
        #[cfg(debug_assertions)]
        log::trace!(
            "Uploading files {:?}",
            files.iter().map(|(name, _)| name).collect::<Vec<_>>()
        );
        let mut form = FormData::new(Vec::new());
        for (name, content) in files {
            form.write_file(
                name,
                Cursor::new(content),
                Some(OsStr::new("file")),
                "application/octet-stream",
            )
            .tap_err(|e| log::debug!("{}", e))
            // The only occasion where in-memory fake I/O can fail is when we run out of memory,
            // and in that case, we're screwed anyway. Having this possible panic here allows us to
            // avoid having a variant for [`std::io::Error`] in our [`Error`] enum.
            .expect("Failed to write file contents to form data");
        }
        let post_body = form
            .finish()
            .tap_err(|e| log::debug!("{}", e))
            .expect("Failed to finish form data"); // Same as above.
        let content_type = form.content_type_header();
        self.make_post_request("upload")
            .header("Content-Type", &content_type)
            .send(&post_body[..])
            .map_err(Error::from)
            .and_then(|res| parse_response::<String>("message", res))
            .tap_ok_dbg(|list| log::trace!("{:?}", list))
            .tap_err(|e| log::debug!("{}", e))
            .and(Ok(()))
    }

    /// Check whether the given path has an allowed extension for this account.
    ///
    /// If the [`free_account`](ClientBuilder::free_account) field is set to `true`, this method
    /// will check that the file extension of the given path is in the list of allowed extensions.
    /// If the field is set to `false`, this method will always return `true`.
    ///
    /// For more information, see <https://neocities.org/site_files/allowed_types>.
    ///
    /// ```
    /// # use neocities_client::{Auth, Client};
    /// assert!(Client::has_allowed_extension(true, "hello.txt"));
    /// assert!(!Client::has_allowed_extension(true, "hello.exe"));
    /// ```
    pub fn has_allowed_extension(free_account: bool, path: &str) -> bool {
        if !free_account {
            true
        } else {
            let unix_path = Utf8UnixPath::new(path);
            let ext = unix_path
                .extension()
                .unwrap_or_default()
                .to_ascii_lowercase();
            ALLOWED_EXTS_FOR_FREE_ACCOUNTS.contains(&ext.as_str())
        }
    }

    // ------------------------------------ Private methods ------------------------------------ //

    /// Build a new GET request with the given path.
    ///
    /// This method will set the appropriate headers, including the `Authorization` header.
    fn make_get_request(&self, path: &str) -> RequestBuilder<WithoutBody> {
        let path = format!("{}/{}", self.base_url, path);
        self.ureq_agent
            .get(&path)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .header("Accept-Charset", "utf-8")
            .header("Authorization", &self.auth.header())
    }

    /// Build a new POST request with the given path.
    ///
    /// This method will set the appropriate headers, including the `Authorization` header.
    fn make_post_request(&self, path: &str) -> RequestBuilder<WithBody> {
        let path = format!("{}/{}", self.base_url, path);
        self.ureq_agent
            .post(&path)
            .header("User-Agent", &self.user_agent)
            .header("Accept", "application/json")
            .header("Accept-Charset", "utf-8")
            .header("Authorization", &self.auth.header())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ErrorKind;
    use indoc::indoc;
    use mockito::{Matcher, Server};

    #[test]
    fn delete_ok() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/delete")
            .match_header("Accept", "application/json")
            .match_header("Accept-Charset", "utf-8")
            .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .match_body(Matcher::UrlEncoded(
                "filenames[]".to_owned(),
                "hello.txt".to_owned(),
            ))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"{ "result": "success", "message": "file(s) have been deleted" }"#)
            .create();
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        client.delete(&["hello.txt"]).unwrap();
        mock.assert();
    }

    #[test]
    fn delete_err() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/delete")
            .match_header("Accept", "application/json")
            .match_header("Accept-Charset", "utf-8")
            .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .match_body(Matcher::UrlEncoded(
                "filenames[]".to_owned(),
                "hello.txt".to_owned(),
            ))
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{
                    "result": "error",
                    "error_type": "missing_files",
                    "message": "img1.jpg was not found on your site, canceled deleting"
                }"#,
            )
            .create();
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        let err = client.delete(&["hello.txt"]).unwrap_err();
        mock.assert();
        assert!(matches!(
            err,
            Error::Api {
                kind: ErrorKind::MissingFiles,
                ..
            }
        ));
    }

    #[test]
    fn info() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/info")
            .match_header("Accept", "application/json")
            .match_header("Accept-Charset", "utf-8")
            .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{
                    "result": "success",
                    "info": {
                        "sitename": "youpi",
                        "views": 235684,
                        "hits": 1487423,
                        "created_at": "Sat, 29 Jun 2013 10:11:38 -0000",
                        "last_updated": "Fri, 01 Dec 2017 18:47:51 -0000",
                        "domain": null,
                        "tags": ["anime", "music", "videogames", "personal", "art"],
                        "latest_ipfs_hash": null
                    }
                }"#,
            )
            .create();
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        let info = client.info().unwrap();
        mock.assert();
        assert_eq!(info.sitename, "youpi");
        assert_eq!(info.views, 235684);
        assert_eq!(info.hits, 1487423);
        assert_eq!(info.created_at, "Sat, 29 Jun 2013 10:11:38 -0000");
        assert_eq!(
            info.last_updated.unwrap(),
            "Fri, 01 Dec 2017 18:47:51 -0000"
        );
        assert_eq!(info.domain, None);
        assert_eq!(
            info.tags,
            vec!["anime", "music", "videogames", "personal", "art"]
        );
        assert_eq!(info.latest_ipfs_hash, None);
    }

    #[test]
    fn key_ok() {
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
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        let key = client.key().unwrap();
        mock.assert();
        assert_eq!(key, "c6275ca833ac06c83926ccb00dff4c82");
    }

    #[test]
    fn key_err() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/key")
            .match_header("Accept", "application/json")
            .match_header("Accept-Charset", "utf-8")
            .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(r#"{
                "result": "error",
                "error_type": "invalid_auth",
                "message": "invalid credentials - please check your username and password (or your api key)"
            }"#)
            .create();
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        let key = client.key().unwrap_err();
        mock.assert();
        assert!(matches!(
            key,
            Error::Api {
                kind: ErrorKind::InvalidAuth,
                ..
            }
        ));
    }

    #[test]
    fn list() {
        let mut server = Server::new();
        let mock = server
            .mock("GET", "/list")
            .match_header("Accept", "application/json")
            .match_header("Accept-Charset", "utf-8")
            .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{
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
                }"#,
            )
            .create();
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        let list = client.list().unwrap();
        mock.assert();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].path, "index.html");
        assert!(!list[0].is_directory);
        assert_eq!(list[0].size, Some(1023));
        assert_eq!(list[0].updated_at, "Sat, 13 Feb 2016 03:04:00 -0000");
        assert_eq!(
            list[0].sha1_hash.clone().unwrap(),
            "c8aac06f343c962a24a7eb111aad739ff48b7fb1"
        );
        assert_eq!(list[1].path, "not_found.html");
        assert!(!list[1].is_directory);
        assert_eq!(list[1].size, Some(271));
        assert_eq!(list[1].updated_at, "Sat, 13 Feb 2016 03:04:00 -0000");
        assert_eq!(
            list[1].sha1_hash.clone().unwrap(),
            "cfdf0bda2557c322be78302da23c32fec72ffc0b"
        );
        assert_eq!(list[2].path, "images");
        assert!(list[2].is_directory);
        assert_eq!(list[2].size, None);
        assert_eq!(list[2].updated_at, "Sat, 13 Feb 2016 03:04:00 -0000");
        assert_eq!(list[2].sha1_hash, None);
        assert_eq!(list[3].path, "images/cat.png");
        assert!(!list[3].is_directory);
        assert_eq!(list[3].size, Some(16793));
        assert_eq!(list[3].updated_at, "Sat, 13 Feb 2016 03:04:00 -0000");
        assert_eq!(
            list[3].sha1_hash.clone().unwrap(),
            "41fe08fc0dd44e79f799d03ece903e62be25dc7d"
        );
    }

    #[test]
    fn upload_ok() {
        let content_type =
            Matcher::Regex("multipart/form-data; boundary=--------+[-A-Za-z0-9_]{32}".to_owned());
        let body = Matcher::Regex(
            indoc! {"
                --------+[-A-Za-z0-9_]{32}\r\n\
                Content-Disposition: form-data; name=\"hello.txt\"; filename=\"file\"\r\n\
                Content-Type: application/octet-stream\r\n\
                \r\n\
                Hello, world!\n\r\n\
                --------+[-A-Za-z0-9_]{32}\r\n\
                Content-Disposition: form-data; name=\"hello1.txt\"; filename=\"file\"\r\n\
                Content-Type: application/octet-stream\r\n\
                \r\n\
                Hello, world!\n\r\n\
                --------+[-A-Za-z0-9_]{32}--\r\n\
            "}
            .to_owned(),
        );
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/upload")
            .match_header("Accept", "application/json")
            .match_header("Accept-Charset", "utf-8")
            .match_header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=")
            .match_header("Content-Type", content_type)
            .match_body(body)
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_body(
                r#"{
                    "result": "success",
                    "message": "your file(s) have been successfully uploaded"
                }"#,
            )
            .create();
        let content = b"Hello, world!\n";
        let client = Client::builder()
            .base_url(server.url())
            .auth(Auth::from("username:password"))
            .build()
            .unwrap();
        client
            .upload(&[("hello.txt", content), ("hello1.txt", content)])
            .unwrap();
        mock.assert();
    }
}
