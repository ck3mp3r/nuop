# Script API Reference

The authoritative reference for the Nushell Operator script protocol: input/output format, exit codes, the configuration object, environment variables, common patterns, best practices, and debugging.

## Input/Output Format

Scripts receive and output resources as JSON via stdin/stdout:

- **Input**: Kubernetes resource as JSON on stdin
- **Output**: Modified resource as JSON on stdout (for reconcile/finalize)
- **Logging**: Use `print` for log messages (goes to stderr)

## Exit Codes

Scripts must exit with specific codes to indicate results:

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success, no changes made |
| 1 | Error occurred |
| 2 | Success, changes made to resource |

## Configuration Object

The `config` function must return a record with these fields:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique name for this operator script |
| `kind` | string | Yes | Kubernetes resource kind to watch |
| `version` | string | Yes | API version of the resource |
| `group` | string | No | API group (for custom resources) |
| `labelSelectors` | record | No | Label selector to filter resources |
| `fieldSelectors` | record | No | Field selector to filter resources |
| `finalizer` | string | No | Finalizer name for cleanup handling |
| `requeue_after_change` | int | No | Requeue interval after changes (default: 10) |
| `requeue_after_noop` | int | No | Requeue interval when no changes (default: 300) |

## Environment Variables

Scripts have access to these environment variables:

| Variable | Description |
|----------|-------------|
| `KUBECONFIG` | Path to Kubernetes config file |
| `NAMESPACE` | Namespace the operator is running in |
| `LOG_LEVEL` | Log level configuration |

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
config_result=$(echo '{}' | nu docs/examples/my-operator/scripts/mod.nu config)
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
kubectl get configmap test-configmap -o json | nu docs/examples/my-operator/scripts/mod.nu reconcile
```

### Local Development

Use the operator's built-in testing capabilities:

```bash
# Run operator in standard mode with your scripts
NUOP_SOURCES_PATH=$PWD/docs/examples op-run-standard

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
echo '{"test": "data"}' | nu docs/examples/my-operator/scripts/mod.nu reconcile

# Check script syntax
nu --ide-check 100 docs/examples/my-operator/scripts/mod.nu

# Run with verbose logging
LOG_LEVEL=debug op-run-standard

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
        version: "v1",
        group: "example.com"  # Specify group for CRDs
    }
}
```

### Multiple Resource Types

Create separate scripts for different resource types and deploy them together in the same operator image.

### Operator Composition

Combine multiple related scripts in a single operator deployment for coordinated resource management.

## Related Documentation

- [Script Development Tutorial](README.md)
- [Examples](examples.md)
- [Deployment Overview](../deployment/README.md)
