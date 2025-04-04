{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nix-filter.url = "github:numtide/nix-filter";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    bomper = {
      url = "github:justinrubek/bomper";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [
        ./flake-parts/shells.nix
        ./flake-parts/factorio.nix
        ./flake-parts/dockerImages.nix
        ./flake-parts/ci.nix

        ./flake-parts/cargo.nix
        ./flake-parts/rust-toolchain.nix

        ./flake-parts/formatting.nix
        ./flake-parts/pre-commit.nix
        inputs.pre-commit-hooks.flakeModule
      ];
    };
}
