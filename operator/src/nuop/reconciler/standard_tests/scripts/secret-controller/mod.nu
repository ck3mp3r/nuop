
# Get configuration for the secret controller
def 'main config' [] {
  {
    name: "secret-controller"
    group: ""
    version: "v1"
    kind: "Secret"
    labelSelectors: {}
    fieldSelectors: {}
    finalizer: "secret.finalizer.example.com"
    namespace: "kube-system"
    requeue_after_change: 25
    requeue_after_noop: 800
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing Secret: ($resource)"
}

# Main help function
def main [] {
  help main
}
