#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: pod-controller
group: ""
version: v1
kind: Pod
labelSelectors:
  environment: production
fieldSelectors:
  status.phase: Running
finalizer: pod.finalizer.example.com
namespace: default
requeue_after_change: 15
requeue_after_noop: 600
EOF
else
    # For actual reconciliation/finalization
    echo "Processing Pod: $1" >&2
    exit 0
fi
