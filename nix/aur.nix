{version, ...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: {
    devshells.aur = {
      packages = [pkgs.git pkgs.pacman];
      commands = [
        {
          help = "Update AUR distribution files";
          name = "update";
          command = let
            makepkgConf = pkgs.writeText "makepkg.conf" ''
              DLAGENTS=('https::${pkgs.curl}/bin/curl -qgb "" -fLC - --retry 3 --retry-delay 3 -o %o %u')
              PKGEXT='.pkg.tar.zst'
              SRCEXT='.src.tar.gz'
            '';
          in
            toString (pkgs.writeShellScript "aur-update" ''
              set -euo pipefail
              IFS=$'\n\t'

              for dir in neocities-deploy{,-bin,-git}; do (
                cd aur/$dir
                case $dir in
                  neocities-deploy|neocities-deploy-bin)
                    sed -i "s/^pkgver=.*/pkgver=${version}/" PKGBUILD
                    sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
                    makepkg --config ${makepkgConf} --geninteg | while read -r line; do
                      var="''${line%%=*}"
                      sed -i "s|^$var=.*|$line|" PKGBUILD
                    done
                    makepkg --config ${makepkgConf} --printsrcinfo > .SRCINFO
                    rm -rf neocities-deploy-* src/
                  ;;
                  neocities-deploy-git)
                    VERSION="$(source PKGBUILD; srcdir=. _pkgname=. GIT_DIR=../../.git pkgver)"
                    sed -i "s/^pkgver=.*/pkgver=$VERSION/" PKGBUILD
                    sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
                    makepkg --config ${makepkgConf} --printsrcinfo > .SRCINFO
                  ;;
                esac
              ); done
            '');
        }
        {
          help = "Push AUR distribution file updates to AUR";
          name = "push";
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
              VERSION="$(source PKGBUILD; echo $pkgver)"
              git commit -m "Update to version $VERSION" \
                && git push origin master \
                || echo "No changes to commit"
              rm -rf .git
            ); done
          '');
        }
      ];
    };
  };
}
