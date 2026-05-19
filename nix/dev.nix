{pkgs, ...}: let
  kind-start = pkgs.writeScriptBin "kind-start" ''
    #!/usr/bin/env nu

    let cluster_name = "nuop"
    let config_path = $"($env.PWD)/kind/kind-cluster.yaml"
    let kube_config_path = $"($env.PWD)/kind/kube.config"

    let clusters = (kind get clusters | lines)
    if ($cluster_name not-in $clusters) {
      print $"Creating kind cluster: ($cluster_name)"
      kind create cluster --name $cluster_name --config $config_path

      let raw_config = (kind get kubeconfig -n $cluster_name)
      $raw_config
      | from yaml
      | reject clusters.0.cluster.certificate-authority-data
      | upsert clusters.0.cluster.insecure-skip-tls-verify true
      | upsert clusters.0.cluster.server "https://127.0.0.1:7543"
      | to yaml
      | save -f $kube_config_path

      print $"Cluster created and kubeconfig saved to ($kube_config_path)"
    } else {
      print $"Cluster ($cluster_name) already exists"
    }
  '';

  op-coverage = pkgs.writeShellScriptBin "op-coverage" "make coverage";
  op-clean = pkgs.writeShellScriptBin "op-clean" "make clean";
  op-crds = pkgs.writeShellScriptBin "op-crds" "make crds";
  op-build = pkgs.writeShellScriptBin "op-build" "make build";
  op-tests = pkgs.writeShellScriptBin "op-tests" "make tests";
  act-test = pkgs.writeShellScriptBin "act-test" "make act-test";
  op-clippy = pkgs.writeShellScriptBin "op-clippy" "make clippy";
  op-fmt = pkgs.writeShellScriptBin "op-fmt" "make fmt";

  op-run-manager = pkgs.writeShellScriptBin "op-run-manager" ''
    cd "$PWD/operator" && LOG_LEVEL=debug NUOP_MODE=manager cargo run --bin operator
  '';

  op-run-standard = pkgs.writeShellScriptBin "op-run-standard" ''
    cd "$PWD/operator" && LOG_LEVEL=debug NUOP_SCRIPT_PATH="$PWD/operator/scripts" cargo run --bin operator
  '';

  op-run-managed = pkgs.writeShellScriptBin "op-run-managed" ''
    cd "$PWD/operator" && LOG_LEVEL=debug NUOP_MODE=managed NUOP_MAPPINGS_PATH="$PWD/operator/test/mappings" NUOP_SCRIPT_PATH="$PWD/operator/scripts" cargo run --bin operator
  '';
in {
  packages = with pkgs;
    [
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
    ]
    ++ [
      kind-start
      op-coverage
      op-clean
      op-crds
      op-build
      op-tests
      act-test
      op-clippy
      op-fmt
      op-run-manager
      op-run-standard
      op-run-managed
    ];

  env = {
    KUBECONFIG = "$PWD/kind/kube.config";
  };

  enterShell = ''
    mkdir -p ./kind
  '';

  shellHook = ''
    lefthook install
  '';
}
