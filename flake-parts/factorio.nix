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
    factorio-pkg = pkgs.factorio-headless.override {
      versionsJson = ../factorio-versions.json;
    };
    factorio-config = pkgs.writeText "factorio.conf" ''
      use-system-read-write-data-directories=true
      [path]
      read-data=${self'.packages.factorio-headless}/share/factorio/data
      write-data=./.factorio
    '';

    factorio-wrapper = pkgs.writeShellApplication {
      name = "factorio";
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
      factorio-headless = factorio-pkg;
      inherit factorio-config factorio-wrapper;
    };
  };
}
