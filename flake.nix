{
  description = "Nushell Operator";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];

      perSystem = {
        system,
        pkgs,
        ...
      }: let
        operatorPackages = import ./operator/nix {
          inherit
            pkgs
            system
            ;
          overlays = [];
          fenix = inputs.fenix;
          nixpkgs = inputs.nixpkgs;
        };
      in {
        packages = operatorPackages;

        devShells.default = inputs.devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            (import ./devenv.nix)
            {
              packages = [
                operatorPackages.toolchain
              ];

              env = {
                RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              };
            }
          ];
        };

        devShells.ci = inputs.devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            (import ./devenv-ci.nix)
            {
              packages = [
                operatorPackages.toolchain
              ];

              env = {
                RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              };
            }
          ];
        };

        formatter = pkgs.alejandra;
      };
    };
}
