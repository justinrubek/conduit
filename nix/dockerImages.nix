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

      "image/conduit" = pkgs.dockerTools.buildImage {
        name = "conduit";
        tag = self.rev or "dirty";

        copyToRoot = [
          self'.packages.conduit
          pkgs.cacert
          pkgs.wget
          pkgs.iproute2
        ];

        config.Cmd = ["/bin/conduit"];
      };
    };
  };
}
