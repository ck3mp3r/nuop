#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: service-controller
group: ""
version: v1
kind: Service
labelSelectors: {}
fieldSelectors: {}
requeue_after_change: 20
requeue_after_noop: 1200
EOF
else
    echo "Processing Service: $1" >&2
    exit 0
fi
