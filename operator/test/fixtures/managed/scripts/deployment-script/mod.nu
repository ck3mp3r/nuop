
# Get configuration for the deployment script
def 'main config' [] {
  {
    name: "deployment-script"
    group: "apps"
    version: "v1"
    kind: "Deployment"
    labelSelectors: {}
    fieldSelectors: {}
    requeue_after_change: 15
    requeue_after_noop: 600
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing: ($resource)"
}

# Main help function
def main [] {
  help main
}
