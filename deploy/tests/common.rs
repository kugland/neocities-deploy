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

#![allow(dead_code)]

use std::{io::Write, path::Path};
use tempfile::NamedTempFile;

/// Build a single-site config file. Kept for backward compatibility with existing tests.
pub fn config_file(auth: &str, path: impl AsRef<Path>) -> NamedTempFile {
    config_file_multi(&[("lorem.com", auth, path.as_ref())])
}

/// Build a config file with one or more sites.
///
/// Each entry is `(name, auth, path)`. Order is preserved.
pub fn config_file_multi(sites: &[(&str, &str, &Path)]) -> NamedTempFile {
    let mut text = String::new();
    for (name, auth, path) in sites {
        text.push_str(&format!(
            "[site.\"{}\"]\nauth = \"{}\"\npath = \"{}\"\n\n",
            name,
            auth,
            path.to_str().unwrap()
        ));
    }
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(text.as_bytes()).unwrap();
    file
}
