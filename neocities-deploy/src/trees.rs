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

use anyhow::{Result, anyhow};
use itertools::Itertools;
use neocities_client::{Client, response::ListEntry};
use sha1::{Digest, Sha1};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::{fs, io};

const NEOCITIES_IGNORE: &str = ".neocitiesignore";

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Path of the entry, relative to the root of the tree.
    pub path: String,
    /// Information about the file, if it is a file.
    pub info: Option<FileInfo>,
    /// Full path to the file on the local file system, if it is local.
    pub local_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    /// Size of the file in bytes.
    pub size: u64,
    /// SHA-1 hash of the file.
    pub sha1_sum: String,
}

impl Entry {
    /// Test whether the entry is a file.
    pub fn is_file(&self) -> bool {
        self.info.is_some()
    }

    /// Check whether two entries have the same content.
    pub fn is_same(&self, other: &Self) -> bool {
        self.info == other.info
    }

    /// Create a new `Entry` from the local file system.
    fn local(root: &Path, entry: &ignore::DirEntry) -> Result<Self> {
        let local_path = entry.path();
        let path = local_path
            .strip_prefix(root)
            .unwrap_or_else(|_| entry.path())
            .to_str()
            .ok_or(anyhow!("Non-UTF-8 path: {:?}", entry.path()))?
            .to_owned();
        let path = if MAIN_SEPARATOR != '/' {
            // Replace Windows path separators with Unix ones.
            path.replace(MAIN_SEPARATOR, "/")
        } else {
            path
        };
        let local_path = Some(local_path.canonicalize()?);
        let metadata = entry.metadata()?;
        let info = if !metadata.is_dir() {
            let size = metadata.len();
            let sha1_sum = {
                let mut hasher = Sha1::new();
                let mut file = fs::File::open(entry.path())?;
                io::copy(&mut file, &mut hasher)?;
                format!("{:x}", hasher.finalize())
            };
            Some(FileInfo { size, sha1_sum })
        } else {
            None
        };
        Ok(Self {
            path,
            local_path,
            info,
        })
    }
}

// Conversion of API’s `ListEntry` to `Entry`.
impl From<&ListEntry> for Entry {
    fn from(entry: &ListEntry) -> Self {
        Self {
            path: entry.path.clone(),
            info: if entry.is_directory {
                None
            } else {
                Some(FileInfo {
                    size: entry.size.expect("Entry has no size"),
                    sha1_sum: (entry.sha1_hash.clone()).expect("Entry has no SHA-1 hash"),
                })
            },
            local_path: None,
        }
    }
}

/// Create a tree from a list of [`ListEntry`] from the API.
pub fn remote_tree(list: &[ListEntry]) -> Vec<Entry> {
    let mut res: Vec<_> = list.iter().map(Entry::from).collect();
    res.sort_by(|a, b| a.path.cmp(&b.path));
    res
}

/// Create a local file tree from a path.
pub fn local_tree(root: impl Into<PathBuf>, free_account: bool) -> Result<Vec<Entry>> {
    let root = root.into().canonicalize()?;

    let walk = ignore::WalkBuilder::new(&root)
        .follow_links(true)
        .same_file_system(false)
        .hidden(false)
        .git_global(false)
        .git_ignore(false)
        .add_custom_ignore_filename(NEOCITIES_IGNORE)
        .build();

    let mut tree: Vec<_> = walk
        .into_iter()
        .map(|e| Entry::local(&root, &e?))
        .filter_ok(|e| !e.path.is_empty())
        .filter_ok(|e| !e.local_path.as_ref().unwrap().ends_with(NEOCITIES_IGNORE))
        .filter_ok(|e| !e.is_file() || Client::has_allowed_extension(free_account, &e.path))
        .try_collect()?;

    tree.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    const HELLO_SHA1: &str = "943a702d06f34599aee1f8da8ef9f7296031d699";
    const GOODBYE_SHA1: &str = "fcb7246c878762b3f752a6e1fc8573f154fffdec";

    fn create_local_tree() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();

        fs::write(root.path().join(NEOCITIES_IGNORE), "ignored").unwrap();
        fs::write(root.path().join("hello"), "Hello, world!").unwrap();
        fs::write(root.path().join("hello.txt"), "Hello, world!").unwrap();

        let subdir = root.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("goodbye"), "Goodbye, world!").unwrap();
        fs::write(subdir.join("ignored"), "Ignored").unwrap();

        let empty = root.path().join("empty");
        fs::create_dir(empty).unwrap();

        root
    }

    #[test]
    fn test_local_tree() {
        let root = create_local_tree();
        let tree = local_tree(root.path(), false).unwrap();
        assert_equal(
            tree.iter().map(|e| &e.path),
            ["empty", "hello", "hello.txt", "subdir", "subdir/goodbye"],
        );
        assert_equal(
            tree.iter().map(|e| e.info.as_ref().map(|i| i.size)),
            [None, Some(13), Some(13), None, Some(15)],
        );
        assert_equal(
            tree.iter()
                .map(|e| e.info.as_ref().map(|i| i.sha1_sum.as_str())),
            [
                None,
                Some(HELLO_SHA1),
                Some(HELLO_SHA1),
                None,
                Some(GOODBYE_SHA1),
            ],
        );
        assert_equal(
            tree.iter().map(|e| e.local_path.clone().unwrap()),
            ["empty", "hello", "hello.txt", "subdir", "subdir/goodbye"]
                .iter()
                .map(|e| root.path().canonicalize().unwrap().join(e)),
        );
        root.close().unwrap();
    }

    #[test]
    fn test_local_tree_free_account() {
        let root = create_local_tree();
        let tree = local_tree(root.path(), true).unwrap();
        assert_equal(
            tree.iter().map(|e| e.path.clone()),
            ["empty", "hello.txt", "subdir"],
        );
        root.close().unwrap();
    }
}
