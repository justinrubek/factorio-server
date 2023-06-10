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
    skopeo-push = pkgs.writeShellScriptBin "skopeo-push" ''
      set -euo pipefail
      # copy an image to a docker registry
      # 1. image - Given as a path to an image archive
      # 2. registry - The registry to push to
      ${pkgs.skopeo}/bin/skopeo copy --insecure-policy "docker-archive:$1" "docker://$2"
    '';
  in {
    apps = {
      skopeo-push = {
        type = "app";
        program = "${skopeo-push}/bin/skopeo-push";
      };
    };

    packages = {
      "scripts/skopeo-push" = skopeo-push;

      "image/factorio-server" = pkgs.dockerTools.buildImage {
        name = "factorio-server";
        tag = self.rev or "dirty";

        copyToRoot = [
          self'.packages.factorio-headless
          pkgs.cacert
          pkgs.iproute2
        ];

        config = {
          Cmd = ["/bin/factorio-server.sh"];

          Volumes = {
            "/factorio" = {};
          };
        };
      };
    };
  };
}
