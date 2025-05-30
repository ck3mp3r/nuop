#!/usr/bin/env nu

# Get configuration for the duplicate pod controller
def 'main config' [] {
  {
    name: "duplicate-pod-controller"
    group: ""
    version: "v1"
    kind: "Pod"
    labelSelectors: {
      environment: "staging"
    }
    fieldSelectors: {}
    requeue_after_change: 10
    requeue_after_noop: 300
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing duplicate Pod: ($resource)"
}

# Main help function
def main [] {
  help main
}
