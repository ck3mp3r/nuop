
# Get configuration for the deployment controller
def 'main config' [] {
  {
    name: "deployment-controller"
    group: "apps"
    version: "v1"
    kind: "Deployment"
    labelSelectors: {
      tier: "frontend"
    }
    fieldSelectors: {}
    finalizer: "deployment.finalizer.example.com"
    requeue_after_change: 30
    requeue_after_noop: 900
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing Deployment: ($resource)"
}

# Main help function
def main [] {
  help main
}
