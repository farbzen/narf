# Installation

## Prerequisites

- [Nix](https://nixos.org) with the `nix-command` and `flakes` features enabled
- a NixOS configuration flake in a git repository

## Flake outputs

| Output                       | Content                                 |
| ---------------------------- | --------------------------------------- |
| `packages.<system>.narf`     | both binaries, `nix run` starts `narf`  |
| `packages.<system>.narf-tui` | same build, `nix run` starts `narf-tui` |
| `packages.<system>.default`  | alias for `narf`                        |
| `overlays.default`           | adds `narf` and `narf-tui` to `pkgs`    |

Supported systems: `x86_64-linux`, `aarch64-linux`.

## Try without installing

```sh
nix run github:farbzen/narf.nix -- --help
nix run github:farbzen/narf.nix#narf-tui
```

## Install on NixOS

```nix
{
  inputs.narf.url = "github:farbzen/narf.nix";

  outputs = {nixpkgs, narf, ...}: {
    nixosConfigurations.<host> = nixpkgs.lib.nixosSystem {
      modules = [
        {
          nixpkgs.overlays = [narf.overlays.default];
        }
        ({pkgs, ...}: {
          environment.systemPackages = [pkgs.narf];
        })
      ];
    };
  };
}
```

> [!NOTE]
> `narf` and `narf-tui` are the same derivation; installing one of them puts
> both binaries on the `PATH`. Both are wrapped with
> [nh](https://github.com/nix-community/nh) and git, which narf calls for the
> rebuild and the commits. nix and nixos-rebuild are taken from the NixOS
> system.

## Binary cache

Prebuilt packages for the locked nixpkgs revision are served from
[farbzen.cachix.org](https://farbzen.cachix.org):

```nix
{
  nix.settings = {
    extra-substituters = ["https://farbzen.cachix.org"];
    extra-trusted-public-keys = ["farbzen.cachix.org-1:8KQga60LYPZVg0E4iKZgQhTmI5dV38Mb9iSuryp8BoY="];
  };
}
```

> [!TIP]
> Cache hits require narf's own nixpkgs input. With
> `inputs.narf.inputs.nixpkgs.follows = "nixpkgs"` the packages are built
> locally.

How the cache is filled is described in [deployment.md](deployment.md).
