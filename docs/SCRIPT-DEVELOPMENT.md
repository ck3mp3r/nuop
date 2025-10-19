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
mkdir -p operator/scripts/my-operator
cd operator/scripts/my-operator
```

### 2. Implement mod.nu

```nushell
# operator/scripts/my-operator/mod.nu

# Configuration function - defines operator behavior
def "main config" [] {
    {
        name: "my-operator",
        kind: "ConfigMap",          # Kubernetes resource kind to watch
        apiVersion: "v1",           # API version of the resource
        labelSelector: {            # Optional: filter resources by labels
            "app.kubernetes.io/managed-by": "my-operator"
        },
        finalizer: "my-operator.example.com/finalizer",
        requeueAfterSeconds: 300    # Requeue interval when no changes
    }
}

# Reconcile function - handles create/update events
def "main reconcile" [] {
    let resource = $in  # Current resource state from Kubernetes
    
    print $"Processing ($resource.metadata.name) in ($resource.metadata.namespace)"
    
    # Your reconciliation logic here
    # Return exit codes:
    # 0 = no changes made
    # 2 = changes made, update resource
    
    # Example: Add an annotation to track processing
    let updated_resource = ($resource | upsert metadata.annotations.processed-at (date now | format date "%Y-%m-%d %H:%M:%S"))
    
    # Output the updated resource to stdout as JSON
    $updated_resource | to json
    
    # Exit with code 2 to indicate changes were made
    exit 2
}

# Finalize function - handles resource deletion
def "main finalize" [] {
    let resource = $in  # Resource being deleted
    
    print $"Finalizing ($resource.metadata.name) in ($resource.metadata.namespace)"
    
    # Cleanup logic here
    # Remove finalizer to allow deletion
    let cleaned_resource = ($resource | upsert metadata.finalizers [])
    
    $cleaned_resource | to json
    exit 2
}
```

### 3. Test Your Script

```bash
# Test configuration
echo '{}' | nu operator/scripts/my-operator/mod.nu config

# Test reconcile with sample resource
echo '{
  "apiVersion": "v1",
  "kind": "ConfigMap", 
  "metadata": {
    "name": "test-cm",
    "namespace": "default",
    "labels": {
      "app.kubernetes.io/managed-by": "my-operator"
    }
  },
  "data": {
    "key": "value"
  }
}' | nu operator/scripts/my-operator/mod.nu reconcile
```

## Script API Reference

### Input/Output Format

Scripts receive and output resources as JSON via stdin/stdout:

- **Input**: Kubernetes resource as JSON on stdin
- **Output**: Modified resource as JSON on stdout (for reconcile/finalize)
- **Logging**: Use `print` for log messages (goes to stderr)

### Exit Codes

Scripts must exit with specific codes to indicate results:

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success, no changes made |
| 1 | Error occurred |
| 2 | Success, changes made to resource |

### Configuration Object

The `config` function must return a record with these fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique name for this operator script |
| `kind` | string | Yes | Kubernetes resource kind to watch |
| `apiVersion` | string | Yes | API version of the resource |
| `group` | string | No | API group (for custom resources) |
| `labelSelector` | record | No | Label selector to filter resources |
| `fieldSelector` | record | No | Field selector to filter resources |
| `finalizer` | string | No | Finalizer name for cleanup handling |
| `requeueAfterSeconds` | int | No | Requeue interval (default: 60) |

### Environment Variables

Scripts have access to these environment variables:

| Variable | Description |
|----------|-------------|
| `KUBECONFIG` | Path to Kubernetes config file |
| `NAMESPACE` | Namespace the operator is running in |
| `RUST_LOG` | Log level configuration |

## Common Patterns

### Resource Replication

Copy resources across namespaces:

```nushell
def "main reconcile" [] {
    let resource = $in
    
    # Get target namespaces from annotation
    let targets = ($resource.metadata.annotations."replicate-to" 
                   | default "" 
                   | split row ",")
    
    for namespace in $targets {
        # Create replica in target namespace
        let replica = ($resource 
                      | upsert metadata.namespace $namespace
                      | upsert metadata.name $"($resource.metadata.name)-replica"
                      | reject metadata.resourceVersion
                      | reject metadata.uid)
        
        # Apply replica (you would use kubectl or Kubernetes API here)
        print $"Would create replica in ($namespace)"
    }
    
    exit 0
}
```

### Condition-Based Processing

Process resources based on specific conditions:

```nushell
def "main reconcile" [] {
    let resource = $in
    
    # Only process resources with specific label
    if ($resource.metadata.labels."process-me"? | default "false") != "true" {
        exit 0
    }
    
    # Check if already processed
    if ("processed" in ($resource.metadata.annotations | default {})) {
        exit 0
    }
    
    # Do processing...
    let updated = ($resource | upsert metadata.annotations.processed "true")
    $updated | to json
    exit 2
}
```

### Error Handling

Handle errors gracefully:

```nushell
def "main reconcile" [] {
    let resource = $in
    
    try {
        # Risky operation
        let result = (some_external_command)
        
        # Success - update resource
        let updated = ($resource | upsert status.result $result)
        $updated | to json
        exit 2
        
    } catch { |err|
        # Log error
        print $"Error processing resource: ($err.msg)"
        
        # Update resource with error status
        let updated = ($resource | upsert status.error $err.msg)
        $updated | to json
        exit 2
    }
}
```

## Testing Custom Scripts

### Unit Testing

Test individual functions:

```bash
# Test configuration returns valid format
config_result=$(echo '{}' | nu operator/scripts/my-operator/mod.nu config)
echo $config_result | jq '.name'  # Should return script name
```

### Integration Testing

Test with real Kubernetes resources:

```bash
# Create test resource
kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-configmap
  labels:
    app.kubernetes.io/managed-by: my-operator
data:
  test: value
EOF

# Get resource and test script
kubectl get configmap test-configmap -o json | nu operator/scripts/my-operator/mod.nu reconcile
```

### Local Development

Use the operator's built-in testing capabilities:

```bash
# Run operator in standard mode with your scripts
NUOP_SCRIPT_PATH=./scripts op-run-standard

# Apply test resources and observe logs
kubectl apply -f test-resources.yaml
kubectl logs -l app.kubernetes.io/name=nuop -f
```

## Best Practices

### Script Organization

- **Keep scripts focused**: One responsibility per script
- **Use helper functions**: Break complex logic into smaller functions
- **Document behavior**: Include README.md explaining what the script does
- **Version your scripts**: Use git tags or version annotations

### Performance

- **Minimize external calls**: Cache data when possible
- **Use efficient selectors**: Narrow label/field selectors to reduce events
- **Handle large resources**: Be mindful of memory usage with large resources

### Error Handling

- **Always handle errors**: Use try/catch for external operations
- **Provide meaningful logs**: Include context in error messages
- **Update resource status**: Reflect errors in resource status fields
- **Use appropriate exit codes**: Return correct codes for operator behavior

### Security

- **Validate inputs**: Check resource structure before processing
- **Limit permissions**: Use minimal RBAC permissions
- **Handle secrets carefully**: Don't log secret data
- **Sanitize outputs**: Ensure output doesn't contain sensitive data

## Debugging Scripts

### Common Issues

1. **Script not found**: Check directory structure and mod.nu exists
2. **Invalid JSON output**: Ensure reconcile/finalize outputs valid JSON
3. **Wrong exit codes**: Use 0, 1, or 2 only
4. **Infinite loops**: Check requeue logic and exit conditions

### Debugging Techniques

```bash
# Test script directly
echo '{"test": "data"}' | nu operator/scripts/my-operator/mod.nu reconcile

# Check script syntax
nu --check operator/scripts/my-operator/mod.nu

# Run with verbose logging
RUST_LOG=debug op-run-standard

# Examine operator logs
kubectl logs -l app.kubernetes.io/name=nuop --tail=100 -f
```

## Advanced Topics

### Custom Resource Definitions

Scripts can work with custom resources:

```nushell
def "main config" [] {
    {
        name: "my-crd-operator",
        kind: "MyCustomResource",
        apiVersion: "v1",
        group: "example.com"  # Specify group for CRDs
    }
}
```

### Multiple Resource Types

Create separate scripts for different resource types and deploy them together in the same operator image.

### Operator Composition

Combine multiple related scripts in a single operator deployment for coordinated resource management.

## Examples

See the [operator/scripts](../operator/scripts/) directory for complete working examples:

- **config-replicator**: Replicates ConfigMaps across namespaces
- **secret-cloner**: Replicates Secrets with targeting controls

These examples demonstrate real-world patterns and can serve as templates for your own operators.