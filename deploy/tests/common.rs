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
