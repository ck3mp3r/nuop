#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: secret-controller
group: ""
version: v1
kind: Secret
labelSelectors: {}
fieldSelectors: {}
finalizer: secret.finalizer.example.com
namespace: kube-system
requeue_after_change: 25
requeue_after_noop: 800
EOF
else
    echo "Processing Secret: $1" >&2
    exit 0
fi
