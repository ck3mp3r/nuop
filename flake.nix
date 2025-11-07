{
  description = "Nushell Operator";

  inputs = {
    base-nixpkgs.url = "github:ck3mp3r/flakes?dir=base-nixpkgs";
    nixpkgs.follows = "base-nixpkgs/unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    rustnix = {
      url = "github:ck3mp3r/flakes?dir=rustnix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin"];

      perSystem = {system, ...}: let
        overlays = [inputs.base-nixpkgs.overlays.default];
        pkgs = import inputs.nixpkgs {inherit system overlays;};

        cargoToml = fromTOML (builtins.readFile ./operator/Cargo.toml);
        cargoLock = {lockFile = ./operator/Cargo.lock;};

        operatorPackages = import ./operator/nix {
          inherit
            inputs
            system
            pkgs
            cargoToml
            cargoLock
            overlays
            ;
        };

        # Import shell configurations
        devShellConfig = import ./nix/dev.nix {inherit pkgs inputs system;};
        ciShellConfig = import ./nix/ci.nix {inherit pkgs inputs system;};

        # Helper to create a shell from config
        mkShellFromConfig = config:
          pkgs.mkShellNoCC {
            packages =
              config.packages
              ++ [
                operatorPackages.toolchain
              ];

            shellHook = ''
              ${config.enterShell}
              ${config.shellHook or ""}
            '';
          }
          // {
            inherit (config) env;
          };
      in {
        packages = operatorPackages.packages;

        devShells.default = mkShellFromConfig (devShellConfig
          // {
            packages = devShellConfig.packages ++ [operatorPackages.toolchain];
          });

        devShells.ci = mkShellFromConfig (ciShellConfig
          // {
            packages = ciShellConfig.packages ++ [operatorPackages.toolchain];
          });

        formatter = pkgs.alejandra;
      };
    };
}
