{pkgs, ...}: {
  packages = with pkgs; [
    openssl
    pkg-config
  ];

  languages = {
    rust.enable = true;
    nix.enable = true;
  };
}
