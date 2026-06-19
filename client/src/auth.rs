////////       This file is part of the source code for neocities-client, a Rust           ////////
////////       library for interacting with the https://neocities.org/ API.                ////////
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

//! This module contains the `Auth` enum, which represents an authentication method for the API.

use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Authentication method for the API, either a username/password pair or an API key.
///
/// ```
/// # use neocities_client::Auth;
/// let credentials = Auth::from("username:password");
/// assert_eq!(credentials.header(), "Basic dXNlcm5hbWU6cGFzc3dvcmQ=");
/// let api_key = Auth::from("api_key");
/// assert_eq!(api_key.header(), "Bearer api_key");
/// ```
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum Auth {
    Credentials(String, String),
    ApiKey(String),
}

impl Auth {
    /// Generate the Authorization HTTP header value for this authentication method.
    pub fn header(&self) -> String {
        match self {
            Auth::Credentials(user, pass) => {
                let val = format!("{}:{}", user, pass);
                format!("Basic {}", BASE64_STANDARD.encode(val))
            }
            Auth::ApiKey(key) => format!("Bearer {}", key),
        }
    }
}

impl<S: AsRef<str>> From<S> for Auth {
    fn from(s: S) -> Self {
        let s = s.as_ref();
        // If the string contains a colon, it's a username/password pair.
        if let Some((user, pass)) = s.split_once(':') {
            Auth::Credentials(user.to_owned(), pass.to_owned())
        } else {
            Auth::ApiKey(s.to_owned())
        }
    }
}

impl From<Auth> for String {
    fn from(auth: Auth) -> Self {
        match auth {
            Auth::Credentials(user, pass) => format!("{}:{}", user, pass),
            Auth::ApiKey(key) => key,
        }
    }
}

impl Debug for Auth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Credentials(username, _) => f
                .debug_tuple("Auth::Credentials")
                .field(username)
                .field(&"********")
                .finish(),
            Self::ApiKey(key) => f
                .debug_tuple("Auth::ApiKey")
                .field(&format!("{}{}", &key[0..6], "*".repeat(32 - 6)))
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth() {
        let credentials = Auth::from("username:password");
        assert_eq!(
            credentials,
            Auth::Credentials("username".to_string(), "password".to_string())
        );
        assert_eq!(credentials.header(), "Basic dXNlcm5hbWU6cGFzc3dvcmQ=");

        let api_key = Auth::from("api_key");
        assert_eq!(api_key, Auth::ApiKey("api_key".to_string()));
        assert_eq!(api_key.header(), "Bearer api_key");
    }

    #[test]
    fn debug_masks_password() {
        let c = Auth::Credentials("alice".into(), "supersecret".into());
        let s = format!("{:?}", c);
        assert!(s.contains("alice"));
        assert!(!s.contains("supersecret"));
        assert_eq!(s, "Auth::Credentials(\"alice\", \"********\")");
    }

    #[test]
    fn debug_masks_api_key() {
        let k = Auth::ApiKey("c6275ca833ac06c83926ccb00dff4c82".into());
        let s = format!("{:?}", k);
        assert!(!s.contains("c6275ca833ac06c83926ccb00dff4c82"));
        // First 6 chars kept, remaining masked with 26 stars (32 - 6).
        assert_eq!(s, "Auth::ApiKey(\"c6275c**************************\")");
    }

    #[test]
    fn into_string_round_trip() {
        let c = Auth::Credentials("u".into(), "p".into());
        assert_eq!(String::from(c.clone()), "u:p");
        assert_eq!(Auth::from(String::from(c.clone())), c);

        let k = Auth::ApiKey("xyz".into());
        assert_eq!(String::from(k.clone()), "xyz");
        assert_eq!(Auth::from(String::from(k.clone())), k);
    }

    #[test]
    fn from_str_no_colon_is_api_key() {
        assert_eq!(Auth::from("plain"), Auth::ApiKey("plain".into()));
    }

    #[test]
    fn from_str_password_with_colon() {
        // Colon in password: only first colon splits.
        assert_eq!(
            Auth::from("user:pa:ss"),
            Auth::Credentials("user".into(), "pa:ss".into())
        );
    }

    #[test]
    fn serde_round_trip() {
        let c = Auth::Credentials("alice".into(), "secret".into());
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"alice:secret\"");
        let de: Auth = serde_json::from_str(&json).unwrap();
        assert_eq!(de, c);

        let k = Auth::ApiKey("abc123".into());
        let json = serde_json::to_string(&k).unwrap();
        assert_eq!(json, "\"abc123\"");
        let de: Auth = serde_json::from_str(&json).unwrap();
        assert_eq!(de, k);
    }
}
