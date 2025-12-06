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

//! This module contains the types used for error handling in this crate.

use parse_display::{Display, FromStr};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    /// Transport errors returned by [`ureq`].
    #[error("Transport error: {0}")]
    Transport(#[from] ureq::Transport),

    /// Errors when deserializing JSON.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API error are the errors returned as part of a JSON response body from the API.
    #[error("API error: {message} ({kind})")]
    Api {
        /// The error message returned by the API.
        message: String,
        /// The kind of error returned by the API.
        kind: ErrorKind,
    },
}

/// Kinds of error returned by the API.
///
/// These errors are not clearly documented by the API, this list is a reverse-engineered list of
/// errors returned by the API.
///
/// [`ErrorKind::Status`] is not returned as part of a JSON response body, but is instead generated
/// when the server returns a 4xx or 5xx status code and we can't parse the response as JSON.
#[derive(Display, FromStr, Debug, PartialEq)]
#[display(style = "snake_case")]
pub enum ErrorKind {
    /// The site you asked for doesn't exist.
    SiteNotFound,
    /// Authentication failed
    InvalidAuth,
    /// You tried to delete `/index.html`.
    CannotDeleteIndex,
    /// You tried to delete `/`.
    CannotDeleteSiteDirectory,
    /// You tried to delete a file that doesn't exist.
    MissingFiles,
    /// You tried to upload a file with a prohibited extension.
    InvalidFileType,
    /// Server returned a 4xx/5xx status code and we couldn't parse the response as JSON.
    #[from_str(ignore)]
    Status,
    /// An unknown error occurred.
    #[from_str(ignore)]
    Unknown,
}

/// The result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;
