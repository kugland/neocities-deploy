{
  perSystem = {
    pkgs,
    config,
    ...
  }: {
    apps = {
      neocities-deploy = {
        type = "app";
        program = "${config.packages.neocities-deploy}/bin/neocities-deploy";
      };
      default = config.apps.neocities-deploy;
    };
  };
}
