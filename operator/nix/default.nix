{
  inputs,
  system,
  pkgs,
  cargoToml,
  cargoLock,
  overlays,
}: let
  supportedTargets = ["x86_64-linux" "aarch64-linux"];

  packages = inputs.rustnix.lib.rust.buildTargetOutputs {
    inherit
      cargoToml
      cargoLock
      overlays
      pkgs
      system
      supportedTargets
      ;
    nixpkgs = inputs.nixpkgs;
    src = ../.;
    packageName = "operator";
    archiveAndHash = false;
    installData = {};
  };

  toolchain = inputs.rustnix.lib.rust.mkToolchain {
    inherit system;
    extras = ["rustfmt" "clippy" "rust-analyzer"];
  };
in {
  inherit packages toolchain;
}
