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
      narf = pkgs.rustPlatform.buildRustPackage {
        pname = "narf";
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

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [pkgs.makeWrapper];

        postInstall = ''
          for program in $out/bin/narf $out/bin/narf-tui; do
            wrapProgram "$program" --prefix PATH : ${lib.makeBinPath [pkgs.git pkgs.nh]}
          done
        '';

        meta = {
          description = "NixOS Advanced Rebuild Flake - Rebuilding nix flakes and creating commits";
          homepage = "https://github.com/farbzen/narf";
          changelog = "https://github.com/farbzen/narf/blob/main/CHANGELOG.md";
          license = lib.licenses.mit;
          platforms = lib.platforms.linux;
          mainProgram = "narf";
          maintainers = [
            {
              email = "dev@farbzen.de";
              github = "farbzen";
              githubId = 60570640;
              name = "Fabian Meier";
            }
          ];
        };
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
