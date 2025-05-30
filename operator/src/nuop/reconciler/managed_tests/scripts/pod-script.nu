#!/usr/bin/env nu

# Get configuration for the pod script
def 'main config' [] {
  {
    name: "pod-script"
    group: ""
    version: "v1"
    kind: "Pod"
    labelSelectors: {
      env: "test"
    }
    fieldSelectors: {
      "metadata.namespace": "default"
    }
    requeue_after_change: 10
    requeue_after_noop: 300
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
