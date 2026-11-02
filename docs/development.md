# Development

## Development environment

```sh
git clone https://github.com/farbzen/narf.git
cd narf
devenv shell              # or: direnv allow
```

[devenv](https://devenv.sh) provides the Nix and Rust tooling; the `.envrc`
loads it automatically with [direnv](https://direnv.net).

## Static analysis and tests

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The unit tests live in a `tests` module at the end of the source file they
cover and test pure logic only: argument parsing, tilde expansion, the commit
message, the TUI's command line, hints, host picker and input selection, and
reading the flake inputs from the lock file. Calls to git, nh and nix as well
as rendering are not covered. `nix build` runs the tests as well.

The workflow [`.github/workflows/rust.yml`](../.github/workflows/rust.yml)
runs the same commands in the devenv shell on pushes to `main` and on pull
requests that touch the Cargo workspace, the devenv files or the workflow.
A build job runs `cargo build --workspace --release --locked` on
`x86_64-linux` and `aarch64-linux` and attaches `narf` and `narf-tui` to the
run as the artifacts `narf-x86_64-linux` and `narf-aarch64-linux`.

> [!NOTE]
> The ELF interpreter of the binaries points to the glibc in the Nix store of
> the devenv shell, so they only run where that store path exists, such as
> NixOS with the nixpkgs revision of `devenv.lock`.

## Build and check

```sh
nix build .#narf          # build the workspace
nix flake check           # evaluate all outputs
nix run nixpkgs#alejandra -- flake.nix package.nix
```

> [!IMPORTANT]
> Flakes only see files tracked by git. Run `git add` on new files before
> `nix build` or `nix flake check`.

## Layout

| Path          | Content                                                      |
| ------------- | ------------------------------------------------------------ |
| `Cargo.toml`  | Cargo workspace with the shared version, edition and license |
| `narf/`       | `narf` crate: library and command-line tool                  |
| `narf-tui/`   | `narf-tui` crate: terminal frontend                          |
| `package.nix` | standalone derivation that fetches a release from GitHub     |
| `flake.nix`   | flake outputs; builds `package.nix` from the local checkout  |

The flake overrides `src`, `version` and `cargoDeps` of `package.nix` with the
local workspace, so `hash` and `cargoHash` in `package.nix` only matter for
builds outside the flake.

## Release

1. Set the new version in `Cargo.toml` and `package.nix`, and move the
   `Unreleased` entries in `CHANGELOG.md` to it.
2. Commit, tag the commit as `v<version>` and push the tag.
3. Set `hash` and `cargoHash` in `package.nix` to `""`, build the derivation
   outside the flake and paste the hashes from the `got:` lines:

   ```sh
   nix build --impure --expr 'with import <nixpkgs> {}; callPackage ./package.nix {}'
   ```

4. Commit the updated `package.nix`.
