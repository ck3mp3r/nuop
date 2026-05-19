# Testing Guide

This guide covers testing strategies specific to the Nushell Operator, focusing on script testing and operator validation.

## Script Testing

### Unit Testing Script Functions

Test each script function independently to ensure they behave correctly:

```bash
# Test configuration function
echo '{}' | nu operator/scripts/my-operator/mod.nu config

# Test reconcile with sample data
cat test-data/configmap.yaml | nu operator/scripts/my-operator/mod.nu reconcile

# Test finalize with deletion scenario
cat test-data/deleting-configmap.yaml | nu operator/scripts/my-operator/mod.nu finalize
```

### Creating Test Data

Create YAML test files for your script testing:

**test-data/configmap.yaml**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-configmap
  namespace: default
  labels:
    app.kubernetes.io/managed-by: my-operator
data:
  key: value
```

**test-data/deleting-configmap.yaml**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-configmap
  namespace: default
  finalizers:
    - my-operator.example.com/finalizer
  deletionTimestamp: "2024-01-01T00:00:00Z"
data:
  key: value
```

### Script Validation

Validate your script configuration meets requirements:

```bash
#!/usr/bin/env nu

# Test script configuration
def test_config [script_path: string] {
    print $"Testing config for ($script_path)"
    let result = (echo '{}' | nu $"($script_path)/mod.nu" config | from yaml)

    # Validate required fields
    if ($result.name | is-empty) {
        error make {msg: "Missing required field: name"}
    }
    if ($result.kind | is-empty) {
        error make {msg: "Missing required field: kind"}
    }
    if ($result.version | is-empty) {
        error make {msg: "Missing required field: version"}
    }

    print "✅ Config test passed"
}

# Test reconcile function behavior
def test_reconcile [script_path: string, test_data: string] {
    print $"Testing reconcile for ($script_path)"
    let result = (cat $test_data | nu $"($script_path)/mod.nu" reconcile | complete)

    # Check exit code (0 or 2 are valid)
    if $result.exit_code not-in [0, 2] {
        error make {msg: $"Invalid exit code: ($result.exit_code)"}
    }

    print "✅ Reconcile test passed"
}
```

## Operator Testing

### Running Tests

Run the Rust test suite:

```bash
op-tests  # Run all unit and integration tests
```

### Testing Deployment Modes

**Standard Mode Testing**:
```bash
# Run operator in standard mode locally
op-run-standard

# Test with sample resources
kubectl apply -f test-data/configmap.yaml
```

**Manager Mode Testing**:
```bash
# Run manager mode locally
op-run-manager

# Apply NuOperator resource
kubectl apply -f docs/examples/basic-example.yaml
```

**Managed Mode Testing**:
```bash
# Run managed mode locally (requires mappings)
op-run-managed
```

### Integration Testing with Kubernetes

Test your scripts against a real Kubernetes cluster:

```bash
# Create test namespace
kubectl create namespace nuop-test

# Apply test resources
kubectl apply -f test-resources.yaml -n nuop-test

# Monitor operator logs
kubectl logs -l app.kubernetes.io/name=nuop -f

# Verify results
kubectl get all -n nuop-test

# Cleanup
kubectl delete namespace nuop-test --ignore-not-found
```

## Testing Best Practices

### 1. Test Script Logic Separately

Test your script functions independently before running them in the operator:

```bash
# Test each function individually
echo '{}' | nu scripts/my-operator/mod.nu config
echo 'test resource yaml' | nu scripts/my-operator/mod.nu reconcile
echo 'deleting resource yaml' | nu scripts/my-operator/mod.nu finalize
```

### 2. Use Representative Test Data

Create test data that represents real scenarios your operator will encounter.

### 3. Validate Exit Codes

Ensure your scripts return correct exit codes:
- `0` - No changes made
- `2` - Changes made, resource should be requeued

### 4. Test Error Conditions

Test how your scripts handle malformed input, missing resources, and edge cases.

## External Testing Resources

For general Kubernetes testing strategies and broader testing approaches, see:
- [Kubernetes Testing Best Practices](https://kubernetes.io/blog/2019/03/22/kubernetes-end-to-end-testing-for-everyone/)
- [Testing Kubernetes Operators](https://sdk.operatorframework.io/docs/building-operators/golang/testing/)
- [Nushell Testing Documentation](https://www.nushell.sh/book/testing.html)