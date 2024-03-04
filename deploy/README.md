# neocities-deploy

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/kugland/neocities-deploy/.github%2Fworkflows%2Fci.yml) ![AUR Version](https://img.shields.io/aur/version/neocities-deploy) ![License](https://img.shields.io/github/license/kugland/neocities-deploy)

---

**neocities-deploy** •
[@codeberg](https://codeberg.org/kugland/neocities-deploy) |
[@github](https://github.com/kugland/neocities-deploy)

**neocities-client** •
[@codeberg](https://codeberg.org/kugland/neocities-client) |
[@github](https://github.com/kugland/neocities-client) |
[@crates.io](https://crates.io/crates/neocities-client) |
[@docs.rs](https://docs.rs/neocities-client)

---

**neocities-deploy** is a command-line tool for deploying your Neocities site.
It can upload files to your site, list remote files, and more.

Also part of this project is a Rust library for interacting with the Neocities
API, **neocities-client**.

This project *is in no way affiliated with Neocities*. It is a personal project
and is not endorsed by Neocities.

## Usage

```neocities-deploy [OPTIONS...] COMMAND```

### Options

* `-c`, `--config-file`: Path of the configuration file.

* `-s`, `--site`: Select a site. (If not given, all sites are selected.)

* `-i`, `--ignore-errors`: Ignore errors and continue.

* `-v`, `--verbose`: More verbosity.

* `-q`, `--quiet`: Less verbosity.

* `-h`, `--help`: Display help message.

* `-V`, `--version`: Display version.

### Commands

* `config`: Configure a site interactively.

* `key`: Replace credentials with API keys in the config file.

* `list`: List files on the site(s).

* `deploy`: Deploy local files to the site(s).

## Configuration

The configuration file is a TOML file. It should be located at
`~/.config/neocities-deploy/config.toml`. A configuration file might look like
this:

```toml
[sites."site1"]
auth = "username:password"
path = "/path/to/site1"
free_account = true

[sites."site2"]
auth = "6f5902ac237024bdd0c176cb93063dc4" # An API key
path = "/path/to/site2"
free_account = false
proxy = "http://localhost:8081"
```

* Only the fields `auth` and `path` are required.

* The `auth` field can be either a username:password pair or an API key. If it
contains a colon, it’s assumed to be a username:password pair.

* Setting `free_account` to `true` will make the tool to ignore file with
extensions not allowed in free accounts when deploying.

## .neocitiesignore

The `.neocitiesignore` file is a text file that specifies files and directories
that will be ignored when deploying. It has the same syntax as `.gitignore` and
works similarly to it: each `.neocitiesignore` file applies to the directory in
which it resides and all its subdirectories.

## Installation

### Windows

It would be best not to use Windows, but if you must, or if you are a masochist,
you like having your data stolen, and you like seeing ads in your operating system,
there are pre-built binaries available on the [releases page](https://github.com/kugland/neocities-deploy/releases/latest).

### macOS

Ditto for macOS. Pre-built binaries are available on the [releases page](https://github.com/kugland/neocities-deploy/releases/latest).

### Arch Linux

The package is available in the AUR as [`neocities-deploy`](https://aur.archlinux.org/packages/neocities-deploy).
You can install it with your favorite AUR helper, for example:

```sh
$ yay -S neocities-deploy
```

### Nix

#### nix-shell

```sh
nix-shell -p 'pkgs.callPackage (builtins.fetchTarball "https://github.com/kugland/neocities-deploy/archive/master.tar.gz") { }'
```

#### NixOS, Home Manager

```nix
# With NixOS
environment.systemPackages = [(pkgs.callPackage (builtins.fetchGit {
  url = "https://github.com/kugland/neocities-deploy";
  ref = "master";
  rev = "3b2e62ef301ce1e7b46ee522d81dd1e7849ee73f"; # master
}) { })];

# For a single user:
users.users.<user>.packages = [ ... ];

# With Home Manager
home.packages = [ ... ];
```

You should, of course, replace `rev` with the latest commit hash.

#### Other distros

You can build the project from source, or use the pre-built static binaries
available on the [releases page](https://github.com/kugland/neocities-deploy/releases/latest) for
a variety of architectures.

## License

This project is licensed under the GNU General Public License v3.0. See the
[LICENSE](LICENSE) file for details.
