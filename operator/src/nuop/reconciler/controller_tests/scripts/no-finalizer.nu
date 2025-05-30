#!/usr/bin/nu --stdin

# Get configuration for the test controller - no finalizer
def 'main config' [] {
  {
    name: "test-controller"
    group: "apps"
    version: "v1"
    kind: "Deployment"
    labelSelectors: {}
    fieldSelectors: {}
    namespace: "default"
    requeue_after_change: 10
    requeue_after_noop: 300
  } | to yaml
}

# Handle reconcile logic
def handle-reconcile [parsed] {
  print $"Reconciling without finalizer: ($parsed.metadata.name)"
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
