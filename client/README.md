# neocities-client

![Crates.io Version](https://img.shields.io/crates/v/neocities-client) ![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/kugland/neocities-client/.github%2Fworkflows%2Fci.yml) ![License](https://img.shields.io/github/license/kugland/neocities-client)

---

**neocities-deploy** •
[GitHub](https://github.com/kugland/neocities-deploy) |
[Codeberg](https://codeberg.org/kugland/neocities-deploy) |
[Releases](https://github.com/kugland/neocities-deploy/releases/latest) |
[AUR](https://aur.archlinux.org/packages/neocities-deploy) |
[AUR (git)](https://aur.archlinux.org/packages/neocities-deploy-git) |
[AUR (binary)](https://aur.archlinux.org/packages/neocities-deploy-bin)

**neocities-client** •
[Codeberg](https://codeberg.org/kugland/neocities-client) |
[GitHub](https://github.com/kugland/neocities-client) |
[crates.io](https://crates.io/crates/neocities-client) |
[docs.rs](https://docs.rs/neocities-client)

---

This is a Rust library for interacting with the [Neocities](https://neocities.org/)
API. It is part of the **neocities-deploy** project, a command-line tool for
deploying a NeoCities site.

This project *is in no way affiliated with Neocities*. It is a personal project
and is not endorsed by Neocities.

## Usage

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

## License

This project is licensed under the GNU General Public License v3.0. See the
[LICENSE](LICENSE) file for details.
