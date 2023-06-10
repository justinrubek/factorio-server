{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    system,
    inputs',
    self',
    ...
  }: let
    ciPackages = [
    ];

    packages = {
    };

    devShells = {
      ci = pkgs.mkShell rec {
        packages = ciPackages;

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
      };
    };
  in rec {
    inherit devShells packages;

    legacyPackages = {
      inherit ciPackages;
    };
  };
}
