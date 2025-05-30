#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    cat << 'EOF'
name: configmap-controller
group: ""
version: v1
kind: ConfigMap
labelSelectors:
  managed: "true"
fieldSelectors:
  metadata.namespace: kube-system
requeue_after_change: 5
requeue_after_noop: 300
EOF
else
    echo "Processing ConfigMap: $1" >&2
    exit 0
fi
