{pkgs, ...}: {
  packages = with pkgs; [
    alejandra
    nushell
    helm-ls
    pre-commit
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
    KUBECONFIG = "$DEVENV_ROOT/kind/kube.config";
  };

  enterShell = ''
    mkdir -p $DEVENV_ROOT/kind
  '';

  scripts = {
    kind-start.exec = ''
      #!/usr/bin/env nu

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
    '';

    op-coverage.exec = "make coverage";
    op-clean.exec = "make clean";
    op-crds.exec = "make crds";
    op-build.exec = "make build";
    op-tests.exec = "make tests";
    act-test.exec = "make act-test";
    op-clippy.exec = "make clippy";
    op-fmt.exec = "make fmt";

    op-run-manager.exec = ''
      cd $DEVENV_ROOT/operator && LOG_LEVEL=debug NUOP_MODE=manager cargo run --bin operator
    '';

    op-run-standard.exec = ''
      cd $DEVENV_ROOT/operator && LOG_LEVEL=debug NUOP_SCRIPT_PATH=$DEVENV_ROOT/operator/scripts cargo run --bin operator
    '';

    op-run-managed.exec = ''
      cd $DEVENV_ROOT/operator && LOG_LEVEL=debug NUOP_MODE=managed NUOP_MAPPINGS_PATH=$DEVENV_ROOT/operator/test/mappings NUOP_SCRIPT_PATH=$DEVENV_ROOT/operator/scripts cargo run --bin operator
    '';
  };

  git-hooks = {
    hooks = {
      alejandra.enable = true;
      check-merge-conflicts.enable = true;
      check-toml.enable = true;
      check-yaml.enable = true;
      end-of-file-fixer.enable = true;
      trim-trailing-whitespace.enable = true;
    };
  };
}
