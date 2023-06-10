{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [
        ./flake-parts/shells.nix
        ./flake-parts/factorio.nix
        ./flake-parts/dockerImages.nix
        ./flake-parts/ci.nix
      ];
    };
}
