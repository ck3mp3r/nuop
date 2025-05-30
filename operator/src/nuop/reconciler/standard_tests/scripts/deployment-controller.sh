#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: deployment-controller
group: apps
version: v1
kind: Deployment
labelSelectors:
  tier: frontend
fieldSelectors: {}
finalizer: deployment.finalizer.example.com
requeue_after_change: 30
requeue_after_noop: 900
EOF
else
    echo "Processing Deployment: $1" >&2
    exit 0
fi
