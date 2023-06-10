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
    inherit (self'.packages) factorio-wrapper;

    devTools = [
      factorio-wrapper
    ];
  in {
    devShells = {
      default = pkgs.mkShell {
        packages = devTools;
      };
    };
  };
}
