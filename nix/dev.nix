{pkgs, ...}: {
  packages = with pkgs; [
    alejandra
    nushell
    helm-ls
    lefthook
    pyright
    black
    colima
    act
    kubernetes-helm
    kind
    kubectl
    tilt
    cargo-tarpaulin
  ];

  env = {
    KUBECONFIG = "$PWD/.direnv/kind/kube.config";
  };

  enterShell = ''
    mkdir -p .direnv/kind
  '';

  shellHook = ''
    # Install lefthook hooks on shell entry
    lefthook install

    # Define shell functions for common operations
    kind-start() {
      nu -c '
        if (kind get clusters | find "nuop" | is-empty) {
          kind create cluster --name nuop --config ./kind/kind-cluster.yaml
          kind get kubeconfig -n nuop
          | from yaml
          | reject clusters.0.cluster.certificate-authority-data
          | upsert clusters.0.cluster.insecure-skip-tls-verify true
          | upsert clusters.0.cluster.server https://127.0.0.1:7543
          | to yaml
          | save -f ./kind/kube.config
          kubectx kind-nuop
        }
      '
    }

    op-coverage() { make coverage; }
    op-clean() { make clean; }
    op-crds() { make crds; }
    op-build() { make build; }
    op-tests() { make tests; }
    act-test() { make act-test; }
    op-clippy() { make clippy; }
    op-fmt() { make fmt; }

    op-run-manager() {
      (cd ./operator && LOG_LEVEL=debug NUOP_MODE=manager cargo run --bin operator)
    }

    op-run-standard() {
      (cd ./operator && LOG_LEVEL=debug NUOP_SCRIPT_PATH=$PWD/operator/scripts cargo run --bin operator)
    }

    op-run-managed() {
      (cd ./operator && LOG_LEVEL=debug NUOP_MODE=managed NUOP_MAPPINGS_PATH=$PWD/operator/test/mappings NUOP_SCRIPT_PATH=$PWD/operator/scripts cargo run --bin operator)
    }

    echo "Development environment loaded."
    echo "Available commands: kind-start, op-coverage, op-clean, op-crds, op-build, op-tests, act-test, op-clippy, op-fmt"
    echo "Available run commands: op-run-manager, op-run-standard, op-run-managed"
  '';
}
