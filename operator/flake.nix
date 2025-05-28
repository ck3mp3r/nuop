{
  description = "Nushell Operator";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    flake-utils,
    fenix,
    nixpkgs,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
        operatorPackages = import ./nix {
          inherit
            fenix
            nixpkgs
            pkgs
            system
            ;
        };
      in {
        packages = operatorPackages;
        formatter = pkgs.alejandra;
      }
    );
}
