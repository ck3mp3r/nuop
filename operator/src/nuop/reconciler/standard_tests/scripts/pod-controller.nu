#!/usr/bin/env nu

# Get configuration for the pod controller
def 'main config' [] {
  {
    name: "pod-controller"
    group: ""
    version: "v1"
    kind: "Pod"
    labelSelectors: {
      environment: "production"
    }
    fieldSelectors: {
      "status.phase": "Running"
    }
    finalizer: "pod.finalizer.example.com"
    namespace: "default"
    requeue_after_change: 15
    requeue_after_noop: 600
  } | to yaml
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing Pod: ($resource)"
}

# Main help function
def main [] {
  help main
}
