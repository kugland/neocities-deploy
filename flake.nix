{
  description = "A command-line tool for deploying your Neocities site";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;}
    {
      systems = [
        "i686-linux"
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = {
        pkgs,
        config,
        ...
      }: {
        packages = {
          neocities-deploy = pkgs.rustPlatform.buildRustPackage {
            pname = "neocities-deploy";
            version = (pkgs.lib.importTOML "${./.}/Cargo.toml").workspace.package.version;
            cargoLock.lockFile = ./Cargo.lock;
            src = pkgs.lib.cleanSource ./.;
            doCheck = false; # Checks are impure due to reliance on network access
            meta = with pkgs.lib; {
              description = "A command-line tool for deploying your Neocities site";
              homepage = "https://github.com/kugland/neocities-deploy";
              license = licenses.gpl3;
              maintainers = [maintainers.kugland];
            };
          };
          default = config.packages.neocities-deploy;
        };
        apps = {
          neocities-deploy = {
            type = "app";
            program = "${config.packages.neocities-deploy}/bin/neocities-deploy";
          };
          default = config.apps.neocities-deploy;
        };
      };
    };
}
