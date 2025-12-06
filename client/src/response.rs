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

//! This module contains the types used to deserialize the JSON responses from the Neocities API.

use crate::{Error, ErrorKind, Result};
use serde::{de::Error as SerdeError, Deserialize};
use serde_json::Value;
use ureq::Response;

/// Type for the response of the `/api/info` endpoint.
///
/// *Note:* the documentation doesn't clearly define which of the following fields are nullable.
/// If any of the fields that are not of the [`Option`] type here happen to come with a null value,
/// we will have a panic situation. This is easily solved by making the offending field optional.
#[derive(Deserialize, Debug)]
pub struct Info {
    /// Name of the site
    pub sitename: String,
    /// Number of views
    pub views: u64,
    /// Number of hits
    pub hits: u64,
    /// Date and time of the creation of the site
    pub created_at: String,
    /// Date and time of the last update of the site (*sometimes not present*)
    pub last_updated: Option<String>,
    /// Optional custom domain (*only for paid accounts*)
    pub domain: Option<String>,
    /// List of tags
    pub tags: Vec<String>,
    /// Latest IPFS hash (*if IPFS archiving is enabled*)
    pub latest_ipfs_hash: Option<String>,
}

/// Type for an item of the array for the response of the `/api/list` endpoint.
///
/// *Note:* This represents a directory entry, which can be either a file or a directory. For
/// files, all fields should be present; for directories, `size` and `sha1_hash` will be absent.
#[derive(Deserialize, Debug)]
pub struct ListEntry {
    /// Path of the file
    pub path: String,
    /// True if the file is a directory, false otherwise
    pub is_directory: bool,
    /// Date and time of the last update of the file
    pub updated_at: String,
    /// Size of the file in bytes (*not present for directories*)
    pub size: Option<u64>,
    /// Hash of the file (*not present for directories*)
    pub sha1_hash: Option<String>,
}

// --------------------------------------------------------------------------------------------- //
//       Beyond this point lie implementation details that are not exported from the crate       //
// --------------------------------------------------------------------------------------------- //

/// Extract a struct representing the API’s response from a HTTP response.
#[allow(clippy::result_large_err)]
pub(crate) fn parse_response<T>(field: &'static str, res: Response) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    /// The basic response structure returned by the API. It contains a `result` field that
    /// indicates whether the request was successful or not, and gives the error kind and
    /// message in case of an error.
    #[derive(Deserialize)]
    #[serde(tag = "result")]
    enum OuterResponse {
        #[serde(rename = "success")]
        Success,
        #[serde(rename = "error")]
        Error {
            error_type: Option<String>,
            message: Option<String>,
        },
    }

    // Save these for later.
    let status = res.status();
    let status_text = res.status_text().to_owned();

    serde_json::from_reader::<_, Value>(res.into_reader()) // First, parse the JSON.
        .map_err(Error::from)
        .and_then(|json| {
            // Let's first try to deserialize the outer response, which contains the type of the
            // response (success or error) and the error type and message in case of an error.
            let outer = serde_json::from_value::<OuterResponse>(json.clone())?;
            match outer {
                OuterResponse::Success => Ok(json), // Pass the JSON object to the next step.
                OuterResponse::Error {
                    // If the response is an error, return an `Error::Api`.
                    error_type,
                    message,
                } => Err(Error::Api {
                    kind: error_type
                        .unwrap_or_default()
                        .parse()
                        .unwrap_or(ErrorKind::Unknown),
                    message: message.unwrap_or("No error message provided".to_owned()),
                }),
            }
        })
        .and_then(|json| {
            // Now that we know the response is successful, let's try to deserialize the inner
            // response, which contains the actual data we want.
            json.get(field)
                .ok_or_else(|| serde_json::Error::missing_field(field))
                .and_then(|v| serde_json::from_value::<T>(v.clone()))
                .map_err(Error::from)
        })
        .map_err(|err| {
            // If we can't parse the error response from the API, return the status instead.
            if matches!(err, Error::Json { .. }) && (400..=599).contains(&status) {
                Error::Api {
                    kind: ErrorKind::Status,
                    message: format!("{} {}", status, status_text),
                }
            } else {
                err
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success() {
        #[derive(Deserialize)]
        struct Foobar {
            foo: String,
            bar: String,
        }
        let res = ureq::Response::new(
            200,
            "OK",
            r#"
                {
                    "result": "success",
                    "foobar": {
                        "foo": "qux",
                        "bar": "baz"
                    },
                    "we": ["don't", "care", "about", "other", "fields"]
                }
            "#,
        )
        .unwrap();
        let foo = parse_response::<Foobar>("foobar", res).unwrap();
        assert_eq!(foo.foo, "qux");
        assert_eq!(foo.bar, "baz");
    }

    #[test]
    fn parse_error() {
        // Here we should get an `Error::Api` with `kind` set to `ErrorKind::InvalidAuth`, since
        // even though we are getting a 401 status code, the response is still a valid JSON object.
        let res = ureq::Response::new(
            401,
            "Unauthorized",
            r#"
                {
                    "result": "error",
                    "error_type": "invalid_auth",
                    "message": "Invalid API key"
                }
            "#,
        )
        .unwrap();
        let err = parse_response::<String>("foobar", res).unwrap_err();
        assert!(matches!(
            err,
            Error::Api {
                kind: ErrorKind::InvalidAuth,
                ..
            }
        ));
    }

    #[test]
    fn parse_invalid_json() {
        // Here we should get an `Error::Json`, since the response is not a valid JSON object, and
        // the status code is not 4xx or 5xx.
        let res = ureq::Response::new(200, "OK", "not json").unwrap();
        let err = parse_response::<String>("foobar", res).unwrap_err();
        assert!(matches!(err, Error::Json { .. }));
    }

    #[test]
    fn parse_invalid_json_error() {
        // Here we should get an `Error::Api` with `kind` set to `ErrorKind::Status`, since the
        // response is not a valid JSON object, and the status code is 4xx or 5xx.
        let res = ureq::Response::new(401, "Unauthorized", "not json").unwrap();
        let err = parse_response::<String>("foobar", res).unwrap_err();
        let Error::Api { message, kind } = err else {
            panic!("Expected an Error::Api {{ .. }}, got {:?}", err);
        };
        assert_eq!(kind, ErrorKind::Status);
        assert_eq!(message, "401 Unauthorized");
    }
}
