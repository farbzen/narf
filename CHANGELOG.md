# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Removed

- Standalone `package.nix`; NARF is now installed only via the flake, which
  builds the derivation itself.

## [2.1.0] - 2026-11-04

### Added

- `narf-tui` sets each option by pressing its flag letter (e.g. `p` toggles
  Pull, `c` edits the comment, `H` opens the host picker); text options now
  show their flag as well.

## [2.0.0] - 2026-11-04

### Changed

- `-n` is now `--no-rebuild` (formerly `--skip-unchanged`): it always skips
  the rebuild and only pulls (with `-p`) and pushes (with `-P`, squashing as
  before) existing commits, leaving uncommitted changes unstaged and
  uncommitted. It can no longer be combined with `-u`, `-U`, `-c`, `-b` or
  `-t`; `narf-tui` hints at these combinations.
- Squashing aborts when changes are already staged instead of adding them to
  the squashed commit.

## [1.0.0] - 2026-11-02

### Added

- `narf` command-line tool that rebuilds a NixOS flake via `nh` and commits
  every new generation with host, generation, NixOS and kernel version.
- Updating all flake inputs (`-u`) or selected ones (`-U`) before a rebuild.
- Pulling before (`-p`) and pushing after (`-P`) a rebuild, squashing unpushed
  rebuild commits unless `-S` is given.
- Dry run (`-d`), build without switching (`-b`), skipping unchanged flakes
  (`-n`), commit comments (`-c`), trace output (`-t`) and non-interactive mode
  (`-y`).
- Saved flake path in `~/.config/narf/flake`; the host defaults to the current
  hostname.
- `narf-tui` terminal frontend with an options form, repository status
  (upstream, unpushed commits, changes, inputs) and NARF running in an
  embedded terminal.
- Nix flake with `narf`, `narf-tui` and `default` packages and an overlay for
  `x86_64-linux` and `aarch64-linux`; `narf` and `narf-tui` bring `nh` and
  `git` on their `PATH`; `nix` and `nixos-rebuild` come from the NixOS
  system.
- MIT license, declared in the Cargo manifests and in the `meta.license` of
  the Nix packages.
- Standalone `package.nix` that fetches a tagged release via
  `fetchFromGitHub`, so NARF can be installed by copying it into a NixOS
  configuration without the flake.
- Unit tests for argument parsing, tilde expansion, the commit message, the
  TUI's command line, hints, host picker and input selection, and reading the
  flake inputs from the lock file; they also run during the Nix build.

[Unreleased]: https://github.com/farbzen/narf/compare/v2.1.0...HEAD
[2.1.0]: https://github.com/farbzen/narf/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/farbzen/narf/compare/v1.0.0...v2.0.0
[1.0.0]: https://github.com/farbzen/narf/releases/tag/v1.0.0
