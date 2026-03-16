{
  perSystem = {
    pkgs,
    config,
    ...
  }: {
    devshells.default = {
      packages = with pkgs; [
        cargo
        rustc
        rust-analyzer
        clippy
        rustfmt
        pkg-config
      ];
      env = [
        {
          name = "RUST_BACKTRACE";
          value = "1";
        }
      ];
      commands = [
        {
          help = "Build the project";
          name = "build";
          command = "cargo build";
        }
      ];
    };
  };
}
