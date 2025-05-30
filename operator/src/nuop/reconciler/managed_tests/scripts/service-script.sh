#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: service-script
group: ""
version: v1
kind: Service
labelSelectors: {}
fieldSelectors: {}
requeue_after_change: 10
requeue_after_noop: 300
EOF
else
    echo "Processing: $1" >&2
    exit 0
fi
