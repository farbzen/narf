# Development

## Development environment

```sh
git clone https://github.com/farbzen/narf.nix.git
cd narf.nix
devenv shell              # or: direnv allow
```

[devenv](https://devenv.sh) provides the Nix tooling; the `.envrc` loads it
automatically with [direnv](https://direnv.net).

## Build and check

```sh
nix build .#narf          # build the workspace
nix flake check           # evaluate all outputs
nix run nixpkgs#alejandra -- flake.nix packages/default.nix
```

> [!IMPORTANT]
> Flakes only see files tracked by git. Run `git add` on new files before
> `nix build` or `nix flake check`.
