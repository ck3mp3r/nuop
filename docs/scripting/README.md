# Script Development Guide

This guide covers how to create custom Nushell operator scripts for the Nushell Operator (nuop).

## Overview

Nushell Operator scripts are organized as directories containing a `mod.nu` entry point file. Each script implements a Kubernetes controller that watches specific resources and executes reconciliation logic written in Nushell.

## Script Structure

### Directory Layout

```
your-operator-script/
├── mod.nu              # Main entry point (required)
├── helpers.nu          # Helper functions (optional)
├── config.nu           # Configuration utilities (optional)
└── README.md           # Documentation (recommended)
```

### Required Functions

Every operator script must implement three main functions in `mod.nu`:

1. **`main config`**: Returns operator configuration and metadata
2. **`main reconcile`**: Handles resource create/update events
3. **`main finalize`**: Handles resource deletion events

## Writing Your First Script

### 1. Create Script Directory

```bash
mkdir -p docs/examples/operators/my-operator
cd docs/examples/operators/my-operator
```

### 2. Implement mod.nu

```nushell
# docs/examples/operators/my-operator/mod.nu

# Configuration function - defines operator behavior
def "main config" [] {
    {
        name: "my-operator",
        group: "",                  # API group (empty for core resources)
        version: "v1",              # API version of the resource
        kind: "ConfigMap",          # Kubernetes resource kind to watch
        labelSelectors: {           # Optional: filter resources by labels
            "app.kubernetes.io/managed-by": "my-operator"
        },
        fieldSelectors: {},         # Optional: filter resources by fields
        finalizer: "my-operator.example.com/finalizer",
        namespace: null,            # Optional: limit to specific namespace
        requeue_after_change: 10,   # Requeue interval after changes (seconds)
        requeue_after_noop: 300     # Requeue interval when no changes (seconds)
    } | to yaml
}

# Reconcile function - handles create/update events
def "main reconcile" [] {
    let resource = ($in | from yaml)  # Parse YAML input from Kubernetes

    print $"Processing ($resource.metadata.name) in ($resource.metadata.namespace)"

    # Your reconciliation logic here
    # Use kubectl commands to make changes
    # Return exit codes:
    # 0 = no changes made
    # 2 = changes made

    # Example: Add a label using kubectl
    let result = (kubectl label configmap $resource.metadata.name processed=true -n $resource.metadata.namespace | complete)

    if $result.exit_code == 0 {
        print "✅ Added processed label"
        exit 2  # Indicate changes were made
    } else {
        print "❌ Failed to update resource"
        exit 0  # No changes made
    }
}

# Finalize function - handles resource deletion
def "main finalize" [] {
    let resource = ($in | from yaml)  # Parse YAML input from Kubernetes

    print $"Finalizing ($resource.metadata.name) in ($resource.metadata.namespace)"

    # Cleanup logic here - remove any managed resources
    # The operator will automatically remove finalizers

    # Example: Delete related configmaps
    kubectl delete configmap -l managed-by=$resource.metadata.name -n $resource.metadata.namespace

    print "✅ Cleanup completed"
    exit 0
}
```

### 3. Test Your Script

```bash
# Test configuration
echo '{}' | nu docs/examples/operators/my-operator/mod.nu config

# Test reconcile with sample resource (YAML format)
echo 'apiVersion: v1
kind: ConfigMap
metadata:
  name: test-cm
  namespace: default
  labels:
    app.kubernetes.io/managed-by: my-operator
data:
  key: value' | nu docs/examples/operators/my-operator/mod.nu reconcile
```

## Next Steps

- [Script API Reference](api-reference.md) — the config/reconcile/finalize protocol, field table, exit codes, environment variables, patterns, and debugging
- [Examples](examples.md) — complete working scripts you can copy
