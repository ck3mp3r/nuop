# Minimal development environment for CI/CD pipelines
# Contains only essential dependencies for running tests
{pkgs, ...}: {
  packages = with pkgs; [
    # Essential for script execution tests
    nushell
  ];

  # Minimal environment setup
  env = {};

  # No shell initialization needed for CI
  enterShell = "";

  # No custom scripts needed for CI
  scripts = {};

  # No git hooks in CI (handled separately)
  git-hooks = {
    hooks = {};
  };
}
