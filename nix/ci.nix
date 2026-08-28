# Minimal development environment for CI/CD pipelines
# Contains only essential dependencies for running tests
{pkgs, ...}: let
  tools = import ./tools.nix {inherit pkgs;};
in {
  packages = with pkgs;
    [
      # Essential for script execution tests
      nushell
    ]
    ++ [
      tools.op-tests
      tools.op-clippy
      tools.op-fmt
      tools.op-crds
    ];

  # Minimal environment setup
  env = {};

  # No shell initialization needed for CI
  enterShell = "";

  # No shell hook needed for CI
  shellHook = "";
}
