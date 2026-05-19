{
  description = "Nushell Operator";

  inputs = {
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

        # Import shell configurations
        devShellConfig = import ./nix/dev.nix {inherit pkgs;};
        ciShellConfig = import ./nix/ci.nix {inherit pkgs;};

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
        packages = operatorPackages;

        devShells.default = mkShellFromConfig (devShellConfig
          // {
            packages = devShellConfig.packages ++ [operatorPackages.toolchain];
            env =
              devShellConfig.env
              // {
                RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              };
          });

        devShells.ci = mkShellFromConfig (ciShellConfig
          // {
            packages = ciShellConfig.packages ++ [operatorPackages.toolchain];
            env =
              ciShellConfig.env
              // {
                RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              };
          });

        formatter = pkgs.alejandra;
      };
    };
}
