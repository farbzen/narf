{pkgs, ...}: {
  packages = with pkgs; [
    go
    hugo
    openssl
    pkg-config
  ];

  languages = {
    rust.enable = true;
    nix.enable = true;
  };
}
