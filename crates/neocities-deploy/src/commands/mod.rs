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

mod config;
mod deploy;
mod key;
mod list;

pub use config::config;
pub use deploy::deploy;
pub use key::key;
pub use list::list;
