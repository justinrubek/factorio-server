{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    inputs',
    self',
    lib,
    ...
  }: let
    inherit (self'.packages) rust-toolchain factorio-wrapper;
    inherit (self'.legacyPackages) cargoExtraPackages ciPackages;

    devTools = [
      factorio-wrapper
      # rust tooling
      rust-toolchain
      pkgs.cargo-audit
      pkgs.cargo-udeps
      pkgs.bacon
      # formatting
      self'.packages.treefmt
    ];
  in {
    devShells = {
      default = pkgs.mkShell rec {
        packages = devTools ++ cargoExtraPackages ++ ciPackages;

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
        RUST_SRC_PATH = "${self'.packages.rust-toolchain}/lib/rustlib/src/rust/src";

        shellHook = ''
          ${config.pre-commit.installationScript}
        '';
      };
    };
  };
}
