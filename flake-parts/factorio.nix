{
  inputs,
  self,
  lib,
  ...
}: {
  imports = [];

  perSystem = {
    self',
    pkgs,
    lib,
    system,
    inputs',
    ...
  }: let
    factorio-config = pkgs.writeText "factorio.conf" ''
      use-system-read-write-data-directories=true
      [path]
      read-data=${self'.packages.factorio-headless}/share/factorio/data
      write-data=./.factorio
    '';

    factorio-wrapper = pkgs.writeShellApplication {
      name = "factorio-server";
      runtimeInputs = [self'.packages.factorio-headless];
      text = ''
        #!/usr/bin/env bash
        exec ${self'.packages.factorio-headless}/bin/factorio \
          --config ${factorio-config} \
          "$@"
      '';
    };
  in {
    _module.args.pkgs = import inputs.nixpkgs {
      inherit system;
      config.allowUnfree = true;
    };

    packages = {
      inherit (pkgs) factorio-headless;
      inherit factorio-config factorio-wrapper;
    };
  };
}
