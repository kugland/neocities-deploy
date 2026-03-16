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

use std::{collections::HashMap, io::Write, path::Path};
use tempfile::NamedTempFile;

pub fn config_file(auth: &str, path: impl AsRef<Path>) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    let path = path.as_ref().to_str().unwrap();

    let mut site_map = HashMap::new();
    site_map.insert("auth", auth);
    site_map.insert("path", path);

    let mut lorem_map = HashMap::new();
    lorem_map.insert("lorem.com", site_map);

    let mut config_map = HashMap::new();
    config_map.insert("site", lorem_map);

    let toml = toml::to_string(&config_map).unwrap();

    file.write_all(toml.as_bytes()).unwrap();
    file
}
