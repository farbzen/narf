{
  description = "NARF - NixOS Advanced Rebuild Flake";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = {
    self,
    nixpkgs,
  }: let
    inherit (nixpkgs) lib;
    forAllSystems = lib.genAttrs ["x86_64-linux" "aarch64-linux"];
    workspace = lib.importTOML ./Cargo.toml;
  in {
    packages = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      narf = (pkgs.callPackage ./package.nix {}).overrideAttrs {
        inherit (workspace.workspace.package) version;
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            ./narf/Cargo.toml
            ./narf/src
            ./narf-tui/Cargo.toml
            ./narf-tui/src
          ];
        };
        cargoDeps = pkgs.rustPlatform.importCargoLock {lockFile = ./Cargo.lock;};
      };
    in {
      inherit narf;
      default = narf;
      narf-tui = narf // {meta = narf.meta // {mainProgram = "narf-tui";};};
    });

    overlays.default = final: _prev: {
      inherit (self.packages.${final.stdenv.hostPlatform.system}) narf narf-tui;
    };
  };
}
