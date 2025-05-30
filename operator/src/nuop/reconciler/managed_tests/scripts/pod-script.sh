#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: pod-script
group: ""
version: v1
kind: Pod
labelSelectors:
  env: test
fieldSelectors:
  metadata.namespace: default
requeue_after_change: 10
requeue_after_noop: 300
EOF
else
    # For actual reconciliation/finalization
    echo "Processing: $1" >&2
    exit 0
fi
