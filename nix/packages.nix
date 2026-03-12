{version, ...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: {
    packages = {
      neocities-deploy = pkgs.rustPlatform.buildRustPackage {
        pname = "neocities-deploy";
        inherit version;
        cargoLock.lockFile = ../Cargo.lock;
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
  };
}
