#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: duplicate-pod-controller
group: ""
version: v1
kind: Pod
labelSelectors:
  environment: staging
fieldSelectors: {}
requeue_after_change: 10
requeue_after_noop: 300
EOF
else
    echo "Processing duplicate Pod: $1" >&2
    exit 0
fi
