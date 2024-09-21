# emancipated

```
/əˈmansəˌpādəd/
adjective

    free from legal, social, or political restrictions; liberated.
```

A certain tools to get something from a specific website to make it free from their platform.

Might changes the name if I figure out a better one.

## Installation

**Requirements**:
- Rust 1.80.0+ (If manually building or using `cargo`)
- 64-bit devices (ARM64/aarch64 support is untested)
- Modern terminal with the following ANSI support:
  - Support [OSC-8](https://github.com/Alhadis/OSC8-Adoption#terminal-emulators)
  - Support [truecolor](https://github.com/termstandard/colors#terminal-emulators) and the standard 8/16 colors
  - Test code: https://gist.github.com/lilydjwg/fdeaf79e921c2f413f44b6f613f6ad53

Go to [Releases](https://github.com/noaione/emancipated-rs/releases) and download the latest release for your platform.

Run it with:

```shell
emancipated --help
```

Or, via nightly channel a.k.a GitHub Actions: Master CI](https://github.com/noaione/emancipated-rs/actions/workflows/ci.yml?query=branch%3Amaster) / [nightly.link](https://nightly.link/noaione/emancipated-rs/workflows/ci/master?preview).

Or, if you want to build it yourself:

```shell
cargo build --locked --release
./target/release/emancipated --help
```

## Usage

You would need to have a pre-existing account registered, then you can authenticate with the tool.

```shell
emancipated auth [email] [password]
```

Then you can use the tool to download the manga.

```shell
emancipated download <manga_slug> -n <volume_number>
```

You could see other available commands by running:

```shell
emancipated --help
```

## License

BSD-3-Clause License, see [LICENSE](LICENSE) for more information.

## Acknowledgement
- neck
- [tosho-mango](https://github.com/noaione/tosho-mango), code structured based on this (and I basically use the same code for the CLI handler)
