# Installation

## Prerequisites

- [Nix](https://nixos.org); the flake outputs need the `nix-command` and
  `flakes` features
- a NixOS configuration flake in a git repository, which NARF rebuilds

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
nix run github:farbzen/narf -- --help
nix run github:farbzen/narf#narf-tui
```

## Install on NixOS

```nix
{
  inputs.narf.url = "github:farbzen/narf";

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
> [nh](https://github.com/nix-community/nh) and git, which NARF calls for the
> rebuild and the commits. nix and nixos-rebuild are taken from the NixOS
> system.

## Install without the flake

[`package.nix`](../package.nix) is a self-contained derivation in nixpkgs
style: it fetches the tagged release with `fetchFromGitHub` and builds it with
the nixpkgs of the importing configuration. Copy it into the configuration,
for example as `narf.nix`, and call it:

```nix
{pkgs, ...}: {
  environment.systemPackages = [(pkgs.callPackage ./narf.nix {})];
}
```

> [!IMPORTANT]
> Copy `package.nix` from `main`, not from a release tag: the hashes of a
> release are committed after its tag.

To switch to another release, change `version` and set `hash` and `cargoHash`
to `""`. The next build fails and prints the correct hashes in its `got:`
lines; paste each in place of its `""`.

> [!NOTE]
> This works without flakes; the build uses the nixpkgs revision of the
> configuration.
