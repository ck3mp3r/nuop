#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: duplicate-script
group: apps
version: v1
kind: Deployment
labelSelectors: {}
fieldSelectors: {}
requeue_after_change: 10
requeue_after_noop: 300
EOF
else
    echo "Processing: $1" >&2
    exit 0
fi
