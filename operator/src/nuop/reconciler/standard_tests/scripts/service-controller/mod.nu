
# Get configuration for the service controller
def 'main config' [] {
  {
    name: "service-controller"
    group: ""
    version: "v1"
    kind: "Service"
    labelSelectors: {}
    fieldSelectors: {}
    requeue_after_change: 20
    requeue_after_noop: 1200
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing Service: ($resource)"
}

# Main help function
def main [] {
  help main
}
