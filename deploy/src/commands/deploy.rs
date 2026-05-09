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

use crate::trees::Entry;
use crate::{params::Params, trees};
use anyhow::Result;
use itertools::{EitherOrBoth::*, Itertools};
use neocities_client::Client;
use parse_display::Display;
use std::fs;

/// Deploy local files to the site(s).
pub fn deploy(params: &Params) -> Result<()> {
    let sites = params.sites()?;
    if sites.is_empty() {
        eprintln!("No sites to deploy");
        return Ok(());
    }
    for (name, site) in sites {
        log::info!("Deploying site: {}", name);
        let free_account = site.free_account.unwrap_or_default();
        let local = trees::local_tree(&site.path, free_account)?;
        let client = site.build_client()?;
        let list = client.list()?;
        let remote = trees::remote_tree(&list);
        for action in Action::make_strategy(local, remote) {
            action.apply(&client).or_else(|e| {
                if params.ignore_errors {
                    log::error!("{}", e);
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
        }
    }
    log::info!("Deployment complete");
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Display)]
/// Actions to deploy the local tree to the site.
pub enum Action {
    /// Upload a file to the remote entry.
    #[display("upload {0.path}")]
    Upload(Entry),
    /// Delete a remote entry.
    #[display("delete remote {0.path}")]
    DeleteRemote(Entry),
}

impl Action {
    /// Apply the action to the client.
    fn apply(&self, client: &Client) -> Result<()> {
        log::info!("Action: {}", self);
        match self {
            Action::Upload(entry) => {
                let local_path = entry.local_path.as_ref().expect("local_path not set");
                let file = fs::read(local_path)?;
                client.upload(&[(&entry.path, &file)])?;
                Ok(())
            }
            Action::DeleteRemote(entry) => {
                client.delete(&[&entry.path])?;
                Ok(())
            }
        }
    }

    /// Compare two file trees and create a strategy to deploy them.
    ///
    /// **Note:** This function assumes that the two trees are sorted by path. Both `local_tree`
    /// and `remote_tree` return sorted trees, so this should be a safe assumption.
    fn make_strategy(local: Vec<Entry>, remote: Vec<Entry>) -> Vec<Action> {
        use Action::*;

        local
            .into_iter()
            .merge_join_by(remote, |a, b| a.path.cmp(&b.path))
            .flat_map(|pair| match pair {
                // Local is a file, remote has no entry: upload.
                Left(l) if l.is_file() => vec![Upload(l)],
                // Local is a directory, remote has no entry: do nothing.
                Left(_) => vec![],
                // Local has no entry, remote is either a file or a directory: delete remote.
                Right(r) => vec![DeleteRemote(r)],
                // Now for the cases where we have both local and remote entries:
                Both(l, r) => match (l.is_file(), r.is_file(), l.is_same(&r)) {
                    // Remote is a file, local is a directory: delete remote.
                    (false, true, _) => vec![DeleteRemote(r)],
                    // Local is a file, remote is a directory: delete remote, upload.
                    (true, false, _) => vec![DeleteRemote(r), Upload(l)],
                    // Both are files, but different: upload.
                    (true, true, false) => vec![Upload(l)],
                    // Otherwise, do nothing.
                    _ => vec![],
                },
            })
            .fold(Vec::new(), |mut acc, action| {
                // After the deletion of a directory, skip the deletion of its children; otherwise,
                // we would get errors because the children would already be gone when we try to
                // delete them.
                match (acc.last(), &action) {
                    (Some(DeleteRemote(last)), DeleteRemote(cur))
                        if cur.path.starts_with(&format!("{}/", last.path)) => {}
                    _ => acc.push(action),
                };
                acc
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trees::FileInfo;
    use std::path::PathBuf;

    fn file(path: &str, sha: &str) -> Entry {
        Entry {
            path: path.to_owned(),
            info: Some(FileInfo {
                size: 1,
                sha1_sum: sha.to_owned(),
            }),
            local_path: Some(PathBuf::from(path)),
        }
    }

    fn dir(path: &str) -> Entry {
        Entry {
            path: path.to_owned(),
            info: None,
            local_path: Some(PathBuf::from(path)),
        }
    }

    #[test]
    fn strategy_empty() {
        assert_eq!(Action::make_strategy(vec![], vec![]), vec![]);
    }

    #[test]
    fn strategy_local_only_file() {
        let local = vec![file("a.txt", "h1")];
        assert_eq!(
            Action::make_strategy(local.clone(), vec![]),
            vec![Action::Upload(local[0].clone())]
        );
    }

    #[test]
    fn strategy_local_only_dir_does_nothing() {
        let local = vec![dir("subdir")];
        assert_eq!(Action::make_strategy(local, vec![]), vec![]);
    }

    #[test]
    fn strategy_remote_only_file() {
        let remote = vec![file("a.txt", "h1")];
        assert_eq!(
            Action::make_strategy(vec![], remote.clone()),
            vec![Action::DeleteRemote(remote[0].clone())]
        );
    }

    #[test]
    fn strategy_remote_only_dir() {
        let remote = vec![dir("subdir")];
        assert_eq!(
            Action::make_strategy(vec![], remote.clone()),
            vec![Action::DeleteRemote(remote[0].clone())]
        );
    }

    #[test]
    fn strategy_same_file_no_action() {
        let local = vec![file("a.txt", "h1")];
        let remote = vec![file("a.txt", "h1")];
        assert_eq!(Action::make_strategy(local, remote), vec![]);
    }

    #[test]
    fn strategy_diff_file_uploads() {
        let local = vec![file("a.txt", "h1")];
        let remote = vec![file("a.txt", "h2")];
        assert_eq!(
            Action::make_strategy(local.clone(), remote),
            vec![Action::Upload(local[0].clone())]
        );
    }

    #[test]
    fn strategy_local_file_remote_dir() {
        let local = vec![file("x", "h1")];
        let remote = vec![dir("x")];
        assert_eq!(
            Action::make_strategy(local.clone(), remote.clone()),
            vec![
                Action::DeleteRemote(remote[0].clone()),
                Action::Upload(local[0].clone()),
            ]
        );
    }

    #[test]
    fn strategy_local_dir_remote_file() {
        let local = vec![dir("x")];
        let remote = vec![file("x", "h1")];
        assert_eq!(
            Action::make_strategy(local, remote.clone()),
            vec![Action::DeleteRemote(remote[0].clone())]
        );
    }

    #[test]
    fn strategy_dedup_children_of_deleted_dir() {
        let remote = vec![
            dir("subdir"),
            file("subdir/a.txt", "h1"),
            file("subdir/nested/b.txt", "h2"),
        ];
        // Local empty: subdir, subdir/a.txt, subdir/nested/b.txt all marked for deletion,
        // but only the parent delete should remain.
        assert_eq!(
            Action::make_strategy(vec![], remote.clone()),
            vec![Action::DeleteRemote(remote[0].clone())]
        );
    }

    #[test]
    fn strategy_dedup_only_when_real_child_path() {
        // "foo" then "foobar" — NOT a child, must not be suppressed.
        let remote = vec![dir("foo"), file("foobar", "h1")];
        assert_eq!(
            Action::make_strategy(vec![], remote.clone()),
            vec![
                Action::DeleteRemote(remote[0].clone()),
                Action::DeleteRemote(remote[1].clone()),
            ]
        );
    }

    #[test]
    fn strategy_mixed() {
        let local = vec![
            file("keep.txt", "h_keep"),
            file("modified.txt", "h_new"),
            file("new.txt", "h_new2"),
            dir("shared_dir"),
            file("shared_dir/inside.txt", "h_inside"),
        ];
        let remote = vec![
            dir("gone_dir"),
            file("gone_dir/orphan.txt", "h_orphan"),
            file("keep.txt", "h_keep"),
            file("modified.txt", "h_old"),
            dir("shared_dir"),
            file("shared_dir/inside.txt", "h_inside"),
        ];
        let expected = vec![
            Action::DeleteRemote(remote[0].clone()), // gone_dir; orphan child suppressed
            Action::Upload(local[1].clone()),        // modified.txt
            Action::Upload(local[2].clone()),        // new.txt
        ];
        assert_eq!(Action::make_strategy(local, remote), expected);
    }

    #[test]
    fn action_display() {
        let f = file("foo/bar.txt", "h");
        assert_eq!(Action::Upload(f.clone()).to_string(), "upload foo/bar.txt");
        assert_eq!(
            Action::DeleteRemote(f).to_string(),
            "delete remote foo/bar.txt"
        );
    }
}
