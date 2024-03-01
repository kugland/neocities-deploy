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

use crate::params::Params;
use anyhow::Result;
use neocities_client::Auth;

/// Replace credentials with API keys in the config file.
pub fn key(params: &Params) -> Result<()> {
    let sites: Vec<_> = (params.sites()?)
        .into_iter()
        .filter(|(_, site)| matches!(site.auth, Auth::Credentials(_, _)))
        .collect();

    if sites.is_empty() {
        eprintln!("No sites to get API keys for.");
        return Ok(());
    }

    let mut config = params.config()?;
    for (name, site) in sites {
        if matches!(site.auth, Auth::ApiKey(_)) {
            continue;
        }
        println!("Getting API key for site {}", name);
        let client = site.build_client()?;
        let key = match client.key() {
            Ok(key) => Ok(key),
            Err(e) => {
                if !params.ignore_errors {
                    Err(e)
                } else {
                    log::error!("{}", e);
                    continue;
                }
            }
        }?;
        config.sites.get_mut(&name).unwrap().auth = Auth::ApiKey(key);
    }
    config.save(params.config_file())?;
    Ok(())
}
