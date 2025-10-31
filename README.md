# neocities-deploy

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/kugland/neocities-deploy/.github%2Fworkflows%2Fci.yml) ![AUR Version](https://img.shields.io/aur/version/neocities-deploy) ![License](https://img.shields.io/github/license/kugland/neocities-deploy)

---

[GitHub](https://github.com/kugland/neocities-deploy) |
[Codeberg](https://codeberg.org/kugland/neocities-deploy) |
[Releases](https://github.com/kugland/neocities-deploy/releases/latest) |
[AUR](https://aur.archlinux.org/packages/neocities-deploy) |
[AUR (git)](https://aur.archlinux.org/packages/neocities-deploy-git) |
[AUR (binary)](https://aur.archlinux.org/packages/neocities-deploy-bin)
[Codeberg](https://codeberg.org/kugland/neocities-deploy) |
[crates.io](https://crates.io/crates/neocities-client) |
[docs.rs](https://docs.rs/neocities-client)

---

**neocities-deploy** is a command-line tool for deploying your Neocities site.
It can upload files to your site, list remote files, and more.

Also part of this project is a Rust library for interacting with the Neocities
API, **neocities-client**.

This project *is in no way affiliated with Neocities*. It is a personal project
and is not endorsed by Neocities.

## neocities-deploy CLI

### Usage

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

### Configuration

The configuration file is a TOML file.

#### Location of the configuration

The location of the configuration file varies with the platform you're
using. (See the documentation for the `config_dir` function in the Rust
package called [directories](https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.config_dir)
for clarification.)

On **Linux**, `$XDG_CONFIG_HOME/neocities-deploy/config.toml`
(or `$HOME/.config/neocities-deploy/config.toml` if `$XDG_CONFIG_HOME` is
not defined). For example: `/home/alice/.config/neocities-deploy/config.toml`.

On **macOS**, `$HOME/Library/Application Support/neocities-deploy/config.toml`.
For example, `/Users/Alice/Library/Application Support/neocities-deploy/config.toml`.

On **Windows**, `{FOLDERID_RoamingAppData}\neocities-deploy\config\config.toml`.
For example, `C:\Users\Alice\AppData\Roaming\neocities-deploy\config\config.toml`.

#### Example configuration

A configuration file might look like this:

```toml
[site."site1"]
auth = "username:password"
path = "/path/to/site1"
free_account = true

[site."site2"]
auth = "6f5902ac237024bdd0c176cb93063dc4" # An API key
path = "/path/to/site2"
free_account = false
proxy = "http://localhost:8081"
```

* Only the fields `auth` and `path` are required.

* The `auth` field can be either a username:password pair or an API key. If it
contains a colon, it's assumed to be a username:password pair.

* Setting `free_account` to `true` will make the tool to ignore file with
extensions not allowed in free accounts when deploying.

### .neocitiesignore

The `.neocitiesignore` file is a text file that specifies files and directories
that will be ignored when deploying. It has the same syntax as `.gitignore` and
works similarly to it: each `.neocitiesignore` file applies to the directory in
which it resides and all its subdirectories.

## neocities-client Library

The `Client` struct provides a simple interface for interacting with the
website API. To use it, first create a new instance of the `Client` struct
(replace `"username:password"` with your actual username and password):

```rust
let client = Client::builder()
    .auth(Auth::from("username:password"))
    .build()?;
```

Once you have a `Client` instance, you can use its methods to interact with the
website API. For example, to create an API key (which can be later used to
authenticate with the API without providing your username and password):

```rust
let api_key = client.key()?;
println!("API key: {}", api_key);
```

Or to get more information about the website:

```rust
let info = client.info()?;
println!("{:?}", info);
```

To list the files on the website:

```rust
let files = client.list()?;
for file in files {
    println!("{}", file.path);
}
```

To upload one or more files to the website:

```rust
client.upload(&[
    ("/1st_file.txt", b"Contents of the first file"),
    ("/2nd_file.txt", b"Contents of the second file"),
])?;
```

To delete one or more files from the website:

```rust
client.delete(&["file1.txt", "file2.txt"])?;
```

For more information on the available methods, see the documentation for the
`Client` struct.

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
nix-shell -p '(pkgs.callPackage (import (builtins.fetchTarball "https://github.com/kugland/nur-packages/tarball/master")) { }).neocities-deploy'
```

#### NixOS, Home Manager

```nix
# With NixOS
environment.systemPackages = [ (pkgs.callPackage (builtins.fetchGit {
  url = "https://github.com/kugland/nur-packages";
  ref = "master";
  rev = "af06c3aa6bd9e350772139b09465ef5228ca514d"; # master
}) {}).neocities-deploy ];

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

## Send me a tip

If you find this project useful, you can send me a tip.

**Bitcoin:** `bc1qlj7jdw6fff0q8yg93ssg6qp04p88cuurgwxk8r`

**Monero:** `43mSMDDTuwbGX8LBH7XpT6gbnUcJ86KWVfrpKbopnk7QTDpQkSb53e43MBGGyZ8FgYZ3YzcaTa4Pb46cQUz3DsXeRn4Ef5e`

