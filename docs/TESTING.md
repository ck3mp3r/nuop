# Testing Guide

This guide covers testing strategies for the Nushell Operator, including unit testing scripts, integration testing with Kubernetes, and end-to-end validation.

## Testing Levels

### 1. Script Unit Testing

Test individual Nushell scripts in isolation.

### 2. Integration Testing  

Test scripts with real Kubernetes resources.

### 3. End-to-End Testing

Test complete operator deployments in Kubernetes clusters.

## Script Unit Testing

### Testing Script Functions

Test each script function independently:

```bash
# Test configuration function
echo '{}' | nu scripts/my-operator/mod.nu config

# Test reconcile with sample data
cat test-data/configmap.json | nu scripts/my-operator/mod.nu reconcile

# Test finalize with deletion scenario
cat test-data/deleting-configmap.json | nu scripts/my-operator/mod.nu finalize
```

### Creating Test Data

Create JSON test files for different scenarios:

```bash
mkdir -p test-data
```

**test-data/configmap.json**:
```json
{
  "apiVersion": "v1",
  "kind": "ConfigMap",
  "metadata": {
    "name": "test-config",
    "namespace": "default",
    "labels": {
      "app.kubernetes.io/replicate": "yes"
    }
  },
  "data": {
    "config.yaml": "test: value"
  }
}
```

**test-data/deleting-configmap.json**:
```json
{
  "apiVersion": "v1",
  "kind": "ConfigMap",
  "metadata": {
    "name": "test-config",
    "namespace": "default",
    "labels": {
      "app.kubernetes.io/replicate": "yes"
    },
    "finalizers": ["my-operator.example.com/finalizer"],
    "deletionTimestamp": "2023-10-19T12:00:00Z"
  },
  "data": {
    "config.yaml": "test: value"
  }
}
```

### Automated Script Testing

Create a test script for automated validation:

**test-scripts.nu**:
```nushell
#!/usr/bin/env nu

# Test script configuration
def test_config [script_path: string] {
    print $"Testing config for ($script_path)"
    let result = (echo '{}' | nu $"($script_path)/mod.nu" config | from json)
    
    # Validate required fields
    if ($result.name | is-empty) {
        error make {msg: "Missing required field: name"}
    }
    if ($result.kind | is-empty) {
        error make {msg: "Missing required field: kind"}
    }
    
    print "✅ Config test passed"
}

# Test script reconcile
def test_reconcile [script_path: string, test_file: string] {
    print $"Testing reconcile for ($script_path) with ($test_file)"
    
    let result = (cat $test_file | nu $"($script_path)/mod.nu" reconcile)
    let exit_code = $env.LAST_EXIT_CODE
    
    if $exit_code not-in [0, 2] {
        error make {msg: $"Invalid exit code: ($exit_code)"}
    }
    
    if $exit_code == 2 {
        # Validate JSON output if changes were made
        $result | from json | ignore
    }
    
    print "✅ Reconcile test passed"
}

# Test all scripts
def main [] {
    let scripts = (glob "scripts/*/mod.nu" | each { path dirname })
    
    for script in $scripts {
        test_config $script
        test_reconcile $script "test-data/configmap.json"
    }
    
    print "🎉 All script tests passed!"
}
```

Run the test suite:
```bash
nu test-scripts.nu
```

### Property-Based Testing

Test scripts with various input scenarios:

**property-test.nu**:
```nushell
# Generate test ConfigMaps with different properties
def generate_test_configmap [name: string, labels: record] {
    {
        apiVersion: "v1",
        kind: "ConfigMap",
        metadata: {
            name: $name,
            namespace: "default",
            labels: $labels
        },
        data: {
            test: "value"
        }
    }
}

# Test script with various label combinations
def test_label_scenarios [script_path: string] {
    let scenarios = [
        {replicate: "yes"},
        {replicate: "no"},
        {app: "test"},
        {},
        {replicate: "yes", environment: "prod"}
    ]
    
    for scenario in $scenarios {
        let test_cm = (generate_test_configmap "test" $scenario)
        print $"Testing with labels: ($scenario)"
        
        $test_cm | to json | nu $"($script_path)/mod.nu" reconcile | ignore
        
        if $env.LAST_EXIT_CODE == 1 {
            print $"❌ Failed with labels: ($scenario)"
        }
    }
}
```

## Integration Testing

### Local Kubernetes Testing

Test scripts with real Kubernetes resources using Kind:

```bash
# Start local cluster
kind-start

# Create test namespace
kubectl create namespace nuop-test

# Apply test resources
kubectl apply -f test-resources.yaml -n nuop-test
```

**test-resources.yaml**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config-1
  namespace: nuop-test
  labels:
    app.kubernetes.io/replicate: "yes"
    test-scenario: "basic"
data:
  config: "test-value-1"
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-config-2
  namespace: nuop-test
  labels:
    app.kubernetes.io/replicate: "yes"
    target-namespaces: "default,kube-system"
data:
  config: "test-value-2"
---
apiVersion: v1
kind: Secret
metadata:
  name: test-secret
  namespace: nuop-test
  labels:
    app.kubernetes.io/replicate: "yes"
type: Opaque
data:
  password: dGVzdC1wYXNzd29yZA==  # test-password
```

### Integration Test Script

**integration-test.nu**:
```nushell
#!/usr/bin/env nu

# Deploy operator in test mode
def deploy_test_operator [] {
    print "🚀 Deploying test operator..."
    
    kubectl apply -f - <<'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nuop-test
  namespace: nuop-test
spec:
  replicas: 1
  selector:
    matchLabels:
      app: nuop-test
  template:
    metadata:
      labels:
        app: nuop-test
    spec:
      containers:
      - name: operator
        image: nuop:local
        env:
        - name: RUST_LOG
          value: debug
        - name: NUOP_MODE
          value: standard
        - name: NUOP_SCRIPT_PATH
          value: /scripts
EOF
}

# Wait for operator to be ready
def wait_for_operator [] {
    print "⏳ Waiting for operator to be ready..."
    
    mut ready = false
    mut attempts = 0
    
    while not $ready and $attempts < 30 {
        let status = (kubectl get pods -l app=nuop-test -n nuop-test -o json 
                     | from json 
                     | get items.0.status.phase? 
                     | default "Unknown")
        
        if $status == "Running" {
            $ready = true
        } else {
            sleep 2sec
            $attempts = $attempts + 1
        }
    }
    
    if not $ready {
        error make {msg: "Operator failed to start"}
    }
    
    print "✅ Operator is ready"
}

# Test resource processing
def test_resource_processing [] {
    print "🧪 Testing resource processing..."
    
    # Wait for reconciliation
    sleep 5sec
    
    # Check if replicas were created
    let replicas = (kubectl get configmaps -A -l "app.kubernetes.io/replicated-by" -o json
                   | from json
                   | get items
                   | length)
    
    if $replicas == 0 {
        error make {msg: "No replicas were created"}
    }
    
    print $"✅ Found ($replicas) replicated resources"
}

# Clean up test resources
def cleanup [] {
    print "🧹 Cleaning up test resources..."
    kubectl delete namespace nuop-test --ignore-not-found
}

# Main test flow
def main [] {
    try {
        deploy_test_operator
        wait_for_operator
        test_resource_processing
        print "🎉 Integration tests passed!"
    } catch { |error|
        print $"❌ Integration test failed: ($error.msg)"
        exit 1
    } finally {
        cleanup
    }
}
```

### Operator Testing in Standard Mode

Test the operator with your scripts:

```bash
# Build operator with your scripts
docker build -t nuop-test .

# Load into Kind cluster
kind load docker-image nuop-test --name nuop

# Run integration tests
nu integration-test.nu
```

## End-to-End Testing

### Multi-Environment Testing

Test across different Kubernetes versions and configurations:

**e2e-test.yaml** (GitHub Actions):
```yaml
name: E2E Tests
on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        k8s-version: ["1.27", "1.28", "1.29"]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Create Kind cluster
      uses: helm/kind-action@v1
      with:
        node_image: kindest/node:v${{ matrix.k8s-version }}.0
        cluster_name: nuop-e2e
    
    - name: Build operator
      run: |
        docker build -t nuop:e2e .
        kind load docker-image nuop:e2e --name nuop-e2e
    
    - name: Run E2E tests
      run: |
        kubectl apply -f test/e2e/
        nu test/e2e-test.nu
```

### Performance Testing

Test operator performance under load:

**load-test.nu**:
```nushell
#!/usr/bin/env nu

# Create multiple test resources
def create_load_test_resources [count: int] {
    print $"Creating ($count) test resources..."
    
    for i in 1..$count {
        kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: load-test-$i
  namespace: default
  labels:
    app.kubernetes.io/replicate: "yes"
    load-test: "true"
data:
  index: "$i"
  timestamp: "$(date now)"
EOF
    }
}

# Monitor reconciliation performance
def monitor_reconciliation [duration: duration] {
    print $"Monitoring for ($duration)..."
    
    let start_time = (date now)
    let end_time = ($start_time + $duration)
    
    mut processed = 0
    
    while (date now) < $end_time {
        let current_replicas = (kubectl get configmaps -A -l "app.kubernetes.io/replicated-by,load-test=true" --no-headers | wc -l)
        $processed = $current_replicas
        print $"Processed: ($processed) replicas"
        sleep 5sec
    }
    
    return $processed
}

# Run load test
def main [--resources=10: int, --duration=60sec: duration] {
    create_load_test_resources $resources
    let result = (monitor_reconciliation $duration)
    
    print $"Load test completed. Processed ($result) replicas in ($duration)"
    
    # Cleanup
    kubectl delete configmaps -l "load-test=true" --all-namespaces
}
```

### Chaos Testing

Test operator resilience:

**chaos-test.nu**:
```nushell
#!/usr/bin/env nu

# Kill operator pod randomly
def chaos_kill_pod [] {
    print "💥 Killing operator pod..."
    kubectl delete pod -l app.kubernetes.io/name=nuop --grace-period=0
    
    # Wait for restart
    sleep 10sec
    
    # Verify it's running again
    kubectl wait --for=condition=Ready pod -l app.kubernetes.io/name=nuop --timeout=60s
}

# Create resource churn
def chaos_resource_churn [] {
    print "🌪️ Creating resource churn..."
    
    # Rapidly create and delete resources
    for i in 1..10 {
        kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: chaos-$i
  labels:
    app.kubernetes.io/replicate: "yes"
data:
  chaos: "true"
EOF
        sleep 1sec
        kubectl delete configmap chaos-$i --ignore-not-found
    }
}

# Run chaos scenarios
def main [] {
    print "🎭 Starting chaos testing..."
    
    chaos_kill_pod
    chaos_resource_churn
    
    # Verify system is stable
    sleep 30sec
    
    let pods = (kubectl get pods -l app.kubernetes.io/name=nuop -o json 
               | from json 
               | get items 
               | where {|pod| $pod.status.phase == "Running"} 
               | length)
    
    if $pods == 0 {
        error make {msg: "Operator not running after chaos test"}
    }
    
    print "✅ Chaos testing completed successfully"
}
```

## Testing Best Practices

### 1. Test Isolation

- Use separate namespaces for tests
- Clean up resources after tests
- Use unique resource names

### 2. Test Data Management

- Store test data in version control
- Use realistic test scenarios
- Test edge cases and error conditions

### 3. Continuous Testing

- Run tests on every commit
- Test against multiple Kubernetes versions
- Include performance regression tests

### 4. Mock External Dependencies

```nushell
# Mock external API calls in tests
def mock_external_api [] {
    # Instead of real HTTP calls, return mock data
    {
        status: "success",
        data: {result: "mocked"}
    }
}
```

### 5. Assertions and Validation

```nushell
# Helper function for test assertions
def assert_equal [actual: any, expected: any, message: string] {
    if $actual != $expected {
        error make {msg: $"($message): expected ($expected), got ($actual)"}
    }
}

# Test example usage
def test_script_output [] {
    let result = (echo '{}' | nu scripts/my-operator/mod.nu config | from json)
    assert_equal $result.name "my-operator" "Script name should match"
    assert_equal $result.kind "ConfigMap" "Should watch ConfigMaps"
}
```

## Testing in CI/CD

### GitHub Actions Example

```yaml
name: Test Scripts
on: [push, pull_request]

jobs:
  test-scripts:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Nix
      uses: DeterminateSystems/nix-installer-action@main
    
    - name: Setup development environment
      run: nix develop --command echo "Environment ready"
    
    - name: Run script unit tests
      run: nix develop --command nu test-scripts.nu
    
    - name: Start Kind cluster
      run: nix develop --command kind-start
    
    - name: Run integration tests
      run: nix develop --command nu integration-test.nu
```

### Local Testing Workflow

```bash
# Development testing workflow
op-tests                    # Run Rust unit/integration tests
nu test-scripts.nu         # Run script unit tests
kind-start                 # Start local cluster
nu integration-test.nu     # Run integration tests
nu load-test.nu --resources=50  # Performance testing
```

## Debugging Test Failures

### Script Debugging

```bash
# Debug script execution with verbose output
echo '{}' | RUST_LOG=debug nu scripts/my-operator/mod.nu config

# Test with different input data
cat test-data/edge-case.json | nu scripts/my-operator/mod.nu reconcile
```

### Operator Debugging

```bash
# Check operator logs during tests
kubectl logs -l app.kubernetes.io/name=nuop -f

# Examine resource states
kubectl get configmaps -o yaml | grep -A5 -B5 "replicated-by"

# Check events for errors
kubectl get events --sort-by='.lastTimestamp'
```

### Test Environment Debugging

```bash
# Verify test environment
kubectl cluster-info
kubectl get nodes
kubectl get ns

# Check resource creation
kubectl get all -n nuop-test
kubectl describe deployment nuop-test -n nuop-test
```

This comprehensive testing approach ensures your Nushell operators are reliable, performant, and production-ready.