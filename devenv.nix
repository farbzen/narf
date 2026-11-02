{pkgs, ...}: {
  environment.systemPackages = with pkgs; [
    openssl
    pkg-config
  ];

  languages = {
    rust.enable = true;
    nix.enable = true;
  };
}
