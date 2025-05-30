#!/usr/bin/env bash

if [ "$1" = "config" ]; then
    echo "invalid yaml: [missing brackets" >&2
    exit 1
else
    echo "Processing: $1" >&2
    exit 0
fi
