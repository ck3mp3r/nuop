{pkgs, ...}: let
  tools = import ./tools.nix {inherit pkgs;};

  cluster-start = pkgs.writeScriptBin "cluster-start" ''
    #!/usr/bin/env nu

    let colima_running = (colima status | complete | get exit_code) == 0
    if not $colima_running {
      print "Starting colima (8 CPUs, 24GB memory)..."
      colima start --cpu 8 --memory 24
    } else {
      print "Colima already running"
    }

    kind-start
  '';

  tilt-up = pkgs.writeScriptBin "tilt-up" ''
    #!/usr/bin/env nu
    cluster-start
    tilt up
  '';

  tilt-down = pkgs.writeScriptBin "tilt-down" ''
    #!/usr/bin/env nu
    tilt down
  '';

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
      prek
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
      cluster-start
      kind-start
      tilt-up
      tilt-down
      tools.op-coverage
      tools.op-clean
      tools.op-crds
      tools.op-build
      tools.op-tests
      tools.act-test
      tools.op-clippy
      tools.op-fmt
      tools.op-buildx
      tools.act-buildx
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
    prek install
  '';
}
