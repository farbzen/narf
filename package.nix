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
  version = "2.1.0";

  src = fetchFromGitHub {
    owner = "farbzen";
    repo = "narf";
    tag = "v${finalAttrs.version}";
    hash = "sha256-+iBDTmgCbJnh6uuexfe5tXY5qLcoE0G7gd8Py4me86k=";
  };

  cargoHash = "sha256-/q6ooiB0cz9ZB7d9asx7fqQKbRtszE1WCUFE8cqQzFE=";

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
    maintainers = [
      {
        email = "dev@farbzen.de";
        github = "farbzen";
        githubId = 60570640;
        name = "Fabian Meier";
      }
    ];
  };
})
