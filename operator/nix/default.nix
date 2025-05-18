{
  fenix,
  nixpkgs,
  pkgs,
  system,
  ...
}: let
  utils = import ./utils.nix;
  toolchain = with fenix.packages.${system};
    combine [
      stable.cargo
      stable.clippy
      stable.rust-analyzer
      stable.rustc
      stable.rustfmt
      targets.aarch64-unknown-linux-musl.stable.rust-std
      targets.x86_64-unknown-linux-musl.stable.rust-std
    ];

  crossPkgs = target: let
    isCrossCompiling = target != system;
    config = utils.getTarget target;
    tmpPkgs = import nixpkgs {
      inherit system;
      crossSystem =
        if isCrossCompiling || pkgs.stdenv.isLinux
        then {
          inherit config;
          rustc = {inherit config;};
          isStatic = pkgs.stdenv.isLinux;
        }
        else null;
    };

    callPackage = pkgs.lib.callPackageWith (tmpPkgs // {inherit config toolchain;});
  in {
    inherit
      callPackage
      ;
    pkgs = tmpPkgs;
  };
in {
  inherit toolchain;
  default = (crossPkgs system).callPackage ./build.nix {};
  operator-x86_64-linux = (crossPkgs "x86_64-linux").callPackage ./build.nix {};
  operator-aarch64-linux = (crossPkgs "aarch64-linux").callPackage ./build.nix {};
}
