# NARF

_NixOS Advanced Rebuild Flake_

NARF rebuilds a NixOS configuration flake and records every rebuild as a git
commit; it is meant for NixOS users who manage their systems with a flake in a
git repository.

A plain `nixos-rebuild` leaves no trace in the configuration repository. NARF
wraps update, rebuild and commit into one step, so the git history of the flake
doubles as a log of every generation built on each host.

- [Installation](installation.md): prerequisites, flake outputs and NixOS
  configuration
- [Usage](usage.md): options, steps of a run and the commit message
- [TUI](tui.md): layout and keys of `narf-tui`
- [Development](development.md): development environment, tests, build,
  documentation and release

The source code is on [GitHub](https://github.com/farbzen/narf).
