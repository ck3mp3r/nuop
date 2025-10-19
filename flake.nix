{
  description = "Nushell Operator";

  inputs = {
    devshell.url = "github:numtide/devshell";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:nixos/nixpkgs";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];

      perSystem = {system, ...}: let
        overlays = [inputs.devshell.overlays.default];
        pkgs = import inputs.nixpkgs {
          inherit system overlays;
        };
        operatorPackages = import ./operator/nix {
          inherit
            overlays
            pkgs
            system
            ;
          fenix = inputs.fenix;
          nixpkgs = inputs.nixpkgs;
        };
      in {
        packages = operatorPackages;

        devShells.default = pkgs.devshell.mkShell {
          packages = [
            operatorPackages.toolchain
          ];
          imports = [
            (pkgs.devshell.importTOML ./devshell.toml)
            "${inputs.devshell}/extra/git/hooks.nix"
          ];
          env = [
            {
              name = "RUST_SRC_PATH";
              value = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            }
          ];
        };

        formatter = pkgs.alejandra;
      };
    };
}
