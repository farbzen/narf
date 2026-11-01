<div align="center">

<img src=".assets/logo.png" alt="narf logo" width="160">

# narf.nix

_NixOS Advanced Rebuild Flake_

[![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](CHANGELOG.md)
[![Nix Flake](https://img.shields.io/badge/nix-flake-5277C3?style=flat-square&logo=nixos&logoColor=white)](flake.nix)
[![Rust](https://img.shields.io/badge/rust-2024-orange?style=flat-square&logo=rust)](packages/Cargo.toml)
[![Cachix](https://img.shields.io/badge/cachix-farbzen-4A90D9?style=flat-square)](https://farbzen.cachix.org)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)

[Overview](#overview) • [Features](#features) •
[Getting Started](#getting-started) • [Documentation](#documentation)

</div>

This repository packages narf as a Nix flake. narf rebuilds a NixOS
configuration flake and records every rebuild as a git commit; it is meant for
NixOS users who manage their systems with a flake in a git repository.

## Overview

The flake builds the Rust workspace under `packages/` and exposes it as
packages and as an overlay for Linux systems (`x86_64-linux`,
`aarch64-linux`). A GitHub Actions workflow keeps nixpkgs up to date and serves
prebuilt binaries from the binary cache
[farbzen.cachix.org](https://farbzen.cachix.org).

The workspace consists of the following applications:

- **narf** – command-line tool that updates, rebuilds and commits a NixOS
  flake, built with [clap](https://docs.rs/clap) and
  [cliclack](https://docs.rs/cliclack). The code lives in `packages/narf/`.
- **narf-tui** – terminal UI frontend for narf, built with
  [ratatui](https://ratatui.rs) and [tui-term](https://docs.rs/tui-term). The
  code lives in `packages/narf-tui/`.

## Features

- **Rebuild as commit**: every new generation is committed with host,
  generation number, NixOS and kernel version in the message.
- **Input updates**: update all flake inputs (`-u`) or selected ones (`-U`)
  before rebuilding.
- **Git sync**: pull before (`-p`) and push after (`-P`) the rebuild, with
  unpushed rebuild commits squashed into one.
- **Safe modes**: dry run (`-d`), build without switching (`-b`) and skipping
  unchanged flakes (`-n`).
- **Remembered flake**: the flake path is saved in `~/.config/narf/flake`, the
  host defaults to the current hostname.
- **TUI**: pick options in a form, see the repository state (upstream,
  unpushed commits, changes, inputs) and run narf in an embedded terminal.

## Getting Started

### Prerequisites

- [Nix](https://nixos.org) with the `nix-command` and `flakes` features enabled
- a NixOS configuration flake in a git repository

### Installation

Try both applications without installing:

```sh
nix run github:farbzen/narf.nix -- --help
nix run github:farbzen/narf.nix#narf-tui
```

To install them on NixOS, add the flake as an input and its overlay to the
configuration:

```nix
inputs.narf.url = "github:farbzen/narf.nix";
# in a NixOS module:
nixpkgs.overlays = [narf.overlays.default];
environment.systemPackages = [pkgs.narf];
```

> [!NOTE]
> `narf` and `narf-tui` are the same derivation; installing one of them puts
> both binaries on the `PATH`.

The full example and the binary cache settings are in
[docs/installation.md](docs/installation.md).

### Usage

```sh
narf --update --push      # update inputs, rebuild, commit and push
narf --dry                # only show what would be done
narf -U nixpkgs -b        # update only nixpkgs, build without switching
narf-tui                  # interactive frontend
```

Without `--flake`, narf reads the flake path from `~/.config/narf/flake` and
offers to save a new `--flake` value there. All options: `narf --help`.

## Documentation

- [Installation, flake outputs and binary cache](docs/installation.md)
- [Development environment, build and checks](docs/development.md)
- [Deployment and binary cache](docs/deployment.md)
- [Changelog](CHANGELOG.md)

## License

See [LICENSE](LICENSE).
