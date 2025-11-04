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
    just
  ];

  enterShell = ''
    export KUBECONFIG="$DEVENV_ROOT/kind/kube.config"
    export GITUB_TOKEN=$(gh auth token)
  '';

  scripts = {
    kind-start.exec = ''
      nu -c "
        let cluster_name = 'nuop'
        let config_path = (\$env.DEVENV_ROOT + '/kind/kind-cluster.yaml')
        let kube_config_path = (\$env.DEVENV_ROOT + '/kind/kube.config')

        # Check if cluster already exists
        let clusters = (kind get clusters | lines)
        if (\$cluster_name not-in \$clusters) {
          print \$'Creating kind cluster: (\$cluster_name)'
          kind create cluster --name \$cluster_name --config \$config_path

          # Get and modify kubeconfig
          let raw_config = (kind get kubeconfig -n \$cluster_name)
          \$raw_config
          | from yaml
          | reject clusters.0.cluster.certificate-authority-data
          | upsert clusters.0.cluster.insecure-skip-tls-verify true
          | upsert clusters.0.cluster.server 'https://127.0.0.1:7543'
          | to yaml
          | save -f \$kube_config_path

          print \$'✓ Cluster created and kubeconfig saved to (\$kube_config_path)'
          print \$'✓ KUBECONFIG is set to: (\$env.KUBECONFIG)'
        } else {
          print \$'✓ Cluster (\$cluster_name) already exists'
        }
      "
    '';

    op-coverage.exec = "just coverage";
    op-clean.exec = "just clean";
    op-crds.exec = "just crds";
    op-build.exec = "just build";
    op-tests.exec = "just tests";
    act-test.exec = "just act-test";
    op-clippy.exec = "just clippy";
    op-fmt.exec = "just fmt";

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
