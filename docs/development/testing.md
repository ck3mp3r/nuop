# Testing Guide

This guide covers the test organization and testing strategies for the Nushell Operator.

## Test Organization

Rust tests live under `operator/test/`, mirroring the `src/` module hierarchy:

```
operator/test/nuop/
├── manager/
│   └── resources/          # Resource generation tests
└── reconciler/
    ├── controller/         # Reconcile, error handling, scenarios
    ├── managed/            # Managed-mode tests
    └── standard/           # Standard-mode tests
```

Test files are declared from their production module with `#[cfg(test)]` and a `#[path]` attribute. No test file may exceed 500 lines; split large files into topical subfiles under a subdirectory of the same name.

## Running Tests

`cargo-nextest` is the test runner. Profiles live in `operator/.config/nextest.toml`.

```bash
# Run all tests (default profile)
op-tests

# Run all tests with the CI profile (JUnit XML output)
op-tests-ci

# Run specific test modules
cargo nextest run --manifest-path operator/Cargo.toml config
cargo nextest run --manifest-path operator/Cargo.toml reconciler
cargo nextest run --manifest-path operator/Cargo.toml manager

# Run tests matching a pattern
cargo nextest run --manifest-path operator/Cargo.toml test_reconcile
```

## Test Categories

- **Unit tests**: Test individual functions and modules
- **Controller tests**: Test full reconciliation loops with a mock Kubernetes API
- **Script-execution tests**: Run real Nushell scripts through the reconciler

### Mock Kubernetes API

Controller tests use `tower_test::mock` to stand in for the Kubernetes API server. A test builds a mock service, spawns a task that answers the expected requests, and asserts the resulting `Action`:

```rust
let (mock_service, mut handle) = mock::pair::<Request<Body>, Response<Body>>();
let client = Client::new(mock_service, "default");
```

### Script-Execution Tests

Tests that run real Nushell scripts are skipped when the `nu` binary is not on `PATH`. Guard them with the `should_skip_script_tests()` helper:

```rust
#[tokio::test]
async fn test_reconcile_active_no_changes() {
    if should_skip_script_tests() {
        return;
    }
    // ...
}
```

## Script Testing

### Unit Testing Script Functions

Test each script function independently to ensure they behave correctly:

```bash
# Test configuration function
echo '{}' | nu docs/examples/operators/my-operator/mod.nu config

# Test reconcile with sample data
cat test-data/configmap.yaml | nu docs/examples/operators/my-operator/mod.nu reconcile

# Test finalize with deletion scenario
cat test-data/deleting-configmap.yaml | nu docs/examples/operators/my-operator/mod.nu finalize
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

## Testing Deployment Modes

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
kubectl apply -f docs/examples/managed/minimal.yaml
```

**Managed Mode Testing**:
```bash
# Run managed mode locally (requires mappings)
op-run-managed
```

## Integration Testing with Kubernetes

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

## Related Documentation

- [Development Guide](README.md)
- [Code Coverage](coverage.md)
- [Contributing](contributing.md)
