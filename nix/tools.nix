{pkgs}: let
  # Shared tools available in both dev and CI shells
  shared = {
    op-tests = pkgs.writeShellScriptBin "op-tests" ''
      cargo test --manifest-path operator/Cargo.toml "$@"
    '';
    op-clippy = pkgs.writeShellScriptBin "op-clippy" ''
      cargo clippy --manifest-path operator/Cargo.toml -- -D warnings
    '';
    op-fmt = pkgs.writeShellScriptBin "op-fmt" ''
      cargo fmt --manifest-path operator/Cargo.toml
    '';
    op-crds = pkgs.writeShellScriptBin "op-crds" ''
      cargo run --manifest-path operator/Cargo.toml --bin generate > operator/chart/crds/nuop.yaml
    '';
  };

  # Dev-only tools
  dev = {
    op-clean = pkgs.writeShellScriptBin "op-clean" ''
      cargo clean --manifest-path operator/Cargo.toml
    '';
    op-coverage = pkgs.writeShellScriptBin "op-coverage" ''
      cargo tarpaulin --manifest-path operator/Cargo.toml --out Html
    '';
    op-build = pkgs.writeShellScriptBin "op-build" ''
      cd operator && docker build --debug -f docker/Dockerfile . -t ghcr.io/ck3mp3r/nuop:latest
      kind load docker-image ghcr.io/ck3mp3r/nuop:latest -n nuop
    '';
    op-buildx = pkgs.writeShellScriptBin "op-buildx" ''
      cd operator && docker buildx create --name mybuilder --use || true
      cd operator && docker buildx inspect --bootstrap
      cd operator && docker buildx build -f docker/Dockerfile --platform linux/amd64,linux/arm64 -t ghcr.io/ck3mp3r/nuop:latest --push .
    '';
    act-test = pkgs.writeShellScriptBin "act-test" ''
      act push \
        --rm \
        --container-options "--network bridge --dns 8.8.8.8 --dns 1.1.1.1" \
        --container-architecture linux/aarch64 \
        -s GITHUB_TOKEN=$GITHUB_TOKEN \
        -s ACTIONS_RUNTIME_TOKEN=$GITHUB_TOKEN \
        -P ubuntu-latest=catthehacker/ubuntu:js-latest \
        -W .github/workflows/test.yaml \
        -j test
    '';
    act-buildx = pkgs.writeShellScriptBin "act-buildx" ''
      act workflow-dispatch \
        --rm \
        --container-architecture linux/aarch64 \
        --privileged \
        --container-daemon-socket /var/run/docker.sock \
        -s GITHUB_TOKEN=$GITHUB_TOKEN \
        -s ACTIONS_RUNTIME_TOKEN=$GITHUB_TOKEN \
        -P ubuntu-latest=catthehacker/ubuntu:js-latest \
        -W .github/workflows/buildx.yaml \
        -j build
    '';
  };
in
  shared // dev
