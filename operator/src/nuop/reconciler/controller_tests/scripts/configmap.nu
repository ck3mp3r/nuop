#!/usr/bin/env nu --stdin

# Get configuration for the test controller - ConfigMap
def 'main config' [] {
  {
    name: "test-controller"
    group: ""
    version: "v1"
    kind: "ConfigMap"
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
  print $"Reconciling ConfigMap: ($parsed.metadata.name)"
  exit 0
}

# Handle finalize logic
def handle-finalize [parsed] {
  print $"Finalizing ConfigMap: ($parsed.metadata.name)"
  exit 0
}

# Process a resource - no changes
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
