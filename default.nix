{
  lib,
  fetchFromGitHub,
  rustPlatform,
}:
with import <nixpkgs>
{
  overlays = [
    (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
  ];
}; let
  rustPlatform = makeRustPlatform {
    cargo = rust-bin.stable.latest.minimal;
    rustc = rust-bin.stable.latest.minimal;
  };

  gitignoreSrc = fetchTarball "https://github.com/hercules-ci/gitignore.nix/archive/master.tar.gz";

  inherit (import gitignoreSrc {inherit (pkgs) lib;}) gitignoreSource;
in
  rustPlatform.buildRustPackage rec {
    pname = "neocities-deploy";
    version = "0.1.10";
    src = gitignoreSource ./.;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    meta = with lib; {
      description = "A command-line tool for deploying your Neocities site.";
      homepage = "https://github.com/kugland/neocities-deploy";
      license = licenses.gpl3;
      maintainers = [];
    };
  }
