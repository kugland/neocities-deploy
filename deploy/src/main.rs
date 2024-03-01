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

mod commands;
mod params;
mod trees;

use anyhow::Result;
use clap::Parser;
use params::{Command, Params};
use std::env;

fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "neocities_deploy");
    }
    pretty_env_logger::try_init()?;

    let params = Params::parse();
    log::set_max_level(params.verbosity());

    match params.command {
        Command::Config => commands::config(&params),
        Command::Key => commands::key(&params),
        Command::List => commands::list(&params),
        Command::Deploy => commands::deploy(&params),
    }?;

    Ok(())
}
