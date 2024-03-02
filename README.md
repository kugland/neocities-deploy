# neocities-deploy

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/kugland/neocities-deploy/.github%2Fworkflows%2Fbuild-and-test.yml) ![License](https://img.shields.io/github/license/kugland/neocities-deploy)

**neocities-deploy** is a command-line tool for deploying your Neocities site.
It can upload files to your site, list remote files, and more.

Also part of this project is a Rust library for interacting with the Neocities
API, [**neocities-client**](https://github.com/kugland/neocities-client).

This project *is in no way affiliated with Neocities*. It is a personal project
and is not endorsed by Neocities.

Repo mirrors at [Codeberg](https://codeberg.org/kugland/neocities-deploy) and
[GitHub](https://github.com/kugland/neocities-deploy).

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

## License

This project is licensed under the GNU General Public License v3.0. See the
[LICENSE](LICENSE) file for details.
