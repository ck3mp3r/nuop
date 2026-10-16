
# Get configuration for the test controller - custom requeue times
def 'main config' [] {
  {
    name: "test-controller"
    group: "apps"
    version: "v1"
    kind: "Deployment"
    labelSelectors: {}
    fieldSelectors: {}
    namespace: "default"
    requeue_after_change: 60
    requeue_after_noop: 600
  } | to yaml
}

# Handle reconcile logic
def handle-reconcile [parsed] {
  print $"Reconciling with custom requeue: ($parsed.metadata.name)"
  exit 0
}

# Process a resource - no changes
def 'main reconcile' [] {
  let parsed = $in | from yaml
  handle-reconcile $parsed
}

# Main help function
def main [] {
  help main
}
