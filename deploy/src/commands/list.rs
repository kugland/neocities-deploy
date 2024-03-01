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

use crate::{params::Params, trees};
use anyhow::Result;
use bytesize::ByteSize;

/// List files on the site(s).
pub fn list(params: &Params) -> Result<()> {
    for (name, site) in params.sites()? {
        println!("Listing site {}", name);
        let client = site.build_client()?;
        let list = client.list().or_else(|e| {
            if params.ignore_errors {
                log::error!("{}", e);
                Ok(vec![])
            } else {
                Err(e)
            }
        })?;
        let remote = trees::remote_tree(&list);
        for entry in remote {
            let (size, path) = if let Some(info) = entry.info {
                (format!("{}", ByteSize(info.size)), entry.path)
            } else {
                ("".to_owned(), format!("{}/", entry.path))
            };
            println!("{:>10}  {}", size, path);
        }
    }
    Ok(())
}
