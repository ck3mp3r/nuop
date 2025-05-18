{
  config,
  toolchain,
  pkgs,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
  (pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  })
  .buildRustPackage {
    name = cargoToml.package.name;
    version = cargoToml.package.version;

    src = ../.;

    cargoLock.lockFile = ../Cargo.lock;
    doCheck = false;

    installPhase = ''
      install -m755 -D target/${config}/release/operator $out/bin/operator
    '';

    meta = {
      description = cargoToml.package.description;
      homepage = cargoToml.package.homepage;
      license = pkgs.lib.licenses.unlicense;
    };
  }
