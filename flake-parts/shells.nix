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
    inherit (self'.packages) factorio-headless;

    devTools = [
      factorio-headless
    ];
  in {
    devShells = {
      default = pkgs.mkShell {
        packages = devTools;
      };
    };
  };
}
