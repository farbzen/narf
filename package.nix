{
  lib,
  rustPlatform,
  fetchFromGitHub,
  makeWrapper,
  git,
  nh,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "narf";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "farbzen";
    repo = "narf";
    tag = "v${finalAttrs.version}";
    hash = "";
  };

  cargoHash = "sha256-y5gnmVLIGHFLN1nf27kJ0SssVdEm9AGGDJda6uYSEps=";

  nativeBuildInputs = [makeWrapper];

  postInstall = ''
    for program in $out/bin/narf $out/bin/narf-tui; do
      wrapProgram "$program" --prefix PATH : ${lib.makeBinPath [git nh]}
    done
  '';

  meta = {
    description = "NixOS Advanced Rebuild Flake - Rebuilding nix flakes and creating commits";
    homepage = "https://github.com/farbzen/narf";
    changelog = "https://github.com/farbzen/narf/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "narf";
  };
})
