
# Get configuration for the configmap controller
def 'main config' [] {
  {
    name: "configmap-controller"
    group: ""
    version: "v1"
    kind: "ConfigMap"
    labelSelectors: {
      managed: "true"
    }
    fieldSelectors: {
      "metadata.namespace": "kube-system"
    }
    requeue_after_change: 5
    requeue_after_noop: 300
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing ConfigMap: ($resource)"
}

# Main help function
def main [] {
  help main
}
