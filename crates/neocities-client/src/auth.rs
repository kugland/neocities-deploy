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

//! This module contains the `Auth` enum, which represents an authentication method for the API.

use base64::{prelude::BASE64_STANDARD, Engine};
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
}
