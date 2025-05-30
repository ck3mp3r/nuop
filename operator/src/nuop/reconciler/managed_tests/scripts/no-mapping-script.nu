#!/usr/bin/env nu

# Get configuration for the no-mapping script
def 'main config' [] {
  {
    name: "no-mapping-script"
    group: "custom"
    version: "v1alpha1"
    kind: "CustomResource"
    labelSelectors: {}
    fieldSelectors: {}
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
