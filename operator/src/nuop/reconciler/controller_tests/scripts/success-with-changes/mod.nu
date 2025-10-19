
# Get configuration for the test controller
def 'main config' [] {
  {
    name: "test-controller"
    group: "apps"
    version: "v1"
    kind: "Deployment"
    labelSelectors: {}
    fieldSelectors: {}
    finalizer: "test.example.com/finalizer"
    namespace: "default"
    requeue_after_change: 10
    requeue_after_noop: 300
  } | to yaml
}

# Handle reconcile logic
def handle-reconcile [parsed] {
  print $"Reconciling: ($parsed.metadata.name) - changes detected"
  exit 2
}

# Handle finalize logic
def handle-finalize [parsed] {
  print $"Finalizing: ($parsed.metadata.name)"
  exit 2
}

# Process a resource - changes detected
def 'main reconcile' [] {
  let parsed = $in | from yaml
  handle-reconcile $parsed
}

# Finalize a resource
def 'main finalize' [] {
  let parsed = $in | from yaml
  handle-finalize $parsed
}

# Main help function
def main [] {
  help main
}
