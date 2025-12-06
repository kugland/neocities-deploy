{
  description = "A command-line tool for deploying your Neocities site";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;}
    {
      imports = [inputs.devshell.flakeModule];
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
      }: let
        version = (pkgs.lib.importTOML "${./.}/Cargo.toml").workspace.package.version;
      in {
        packages = {
          neocities-deploy = pkgs.rustPlatform.buildRustPackage {
            pname = "neocities-deploy";
            inherit version;
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
        legacyPackages = {
          neocities-deploy = config.packages.neocities-deploy;
          default = config.packages.default;
        };
        apps = {
          neocities-deploy = {
            type = "app";
            program = "${config.packages.neocities-deploy}/bin/neocities-deploy";
          };
          default = config.apps.neocities-deploy;
        };
        devshells.default = {
          commands = [
            {
              help = "Update AUR distribution files";
              name = "aur-update";
              command = let
                makepkgConf = pkgs.writeText "makepkg.conf" ''
                  DLAGENTS=('https::${pkgs.curl}/bin/curl -qgb "" -fLC - --retry 3 --retry-delay 3 -o %o %u')
                  PKGEXT='.pkg.tar.zst'
                  SRCEXT='.src.tar.gz'
                '';
              in
                toString (pkgs.writeShellScript "aur-update.sh" ''
                  set -euo pipefail
                  IFS=$'\n\t'

                  for dir in neocities-deploy{,-bin,-git}; do (
                    cd aur/$dir
                    case $dir in
                      neocities-deploy|neocities-deploy-bin)
                        sed -i "s/^pkgver=.*/pkgver=${version}/" PKGBUILD
                        sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
                        ${pkgs.pacman}/bin/makepkg --config ${makepkgConf} --geninteg | while read -r line; do
                          var="''${line%%=*}"
                          sed -i "s|^$var=.*|$line|" PKGBUILD
                        done
                        ${pkgs.pacman}/bin/makepkg --config ${makepkgConf} --printsrcinfo > .SRCINFO
                        rm -rf neocities-deploy-* src/
                      ;;
                      neocities-deploy-git)
                        VERSION="$( source PKGBUILD; srcdir=. _pkgname=. GIT_DIR=../../.git pkgver )"
                        sed -i "s/^pkgver=.*/pkgver=$VERSION/" PKGBUILD
                        sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
                        ${pkgs.pacman}/bin/makepkg --config ${makepkgConf} --printsrcinfo > .SRCINFO
                      ;;
                    esac
                  ); done
                '');
            }
            {
              help = "Push AUR distribution file updates to AUR";
              name = "aur-push";
              command = toString (pkgs.writeShellScript "aur-push" ''
                set -euo pipefail
                IFS=$'\n\t'

                for dir in neocities-deploy{,-bin,-git}; do (
                  rm -rf aur/$dir/.git
                  git clone --bare --depth 1 ssh://aur.archlinux.org/$dir.git aur/$dir/.git
                  cd aur/$dir
                  export GIT_DIR=.git
                  export GIT_WORK_TREE=.
                  git add PKGBUILD .SRCINFO
                  git commit -m "Update to version ${version}"
                  git push origin master
                  rm -rf .git
                ); done
              '');
            }
          ];
        };
      };
    };
}
