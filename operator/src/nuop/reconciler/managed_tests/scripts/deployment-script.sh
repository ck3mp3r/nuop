#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: deployment-script
group: apps
version: v1
kind: Deployment
labelSelectors: {}
fieldSelectors: {}
requeue_after_change: 15
requeue_after_noop: 600
EOF
else
    echo "Processing: $1" >&2
    exit 0
fi
