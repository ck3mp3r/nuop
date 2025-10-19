# NuOperator Examples

This directory contains working examples for the NuOperator CRD that demonstrate real functionality using actual scripts from the nuop repository.

## Available Examples

### 1. config-replicator.yaml
**Purpose**: Demonstrates ConfigMap replication across namespaces using the built-in config-replicator script.

**Features shown**:
- Manager + Managed mode deployment
- Real script usage (`config-replicator` from nuop repository)
- Proper RBAC configuration for ConfigMap operations
- Label-based resource selection
- Complete example with test ConfigMap

**Usage**:
```bash
kubectl apply -f config-replicator.yaml

# Test by creating a ConfigMap with the replication label
# (Example ConfigMap is included in the file)
```

### 2. secret-cloner.yaml  
**Purpose**: Demonstrates Secret cloning across namespaces using the built-in secret-cloner script.

**Features shown**:
- Secret management operations
- Cross-namespace secret replication
- Proper RBAC for Secret operations
- Complete example with test Secret

**Usage**:
```bash
kubectl apply -f secret-cloner.yaml

# Test by creating a Secret with the replication label
# (Example Secret is included in the file)
```

### 3. minimal.yaml
**Purpose**: The simplest possible NuOperator configuration for learning the basics.

**Features shown**:
- Minimal required fields only
- No RBAC (uses default service account)
- Basic mapping configuration

**Usage**:
```bash
kubectl apply -f minimal.yaml
```

### 4. local-development.yaml
**Purpose**: Shows how to develop and test scripts locally using volume mounts.

**Features shown**:
- Local script development workflow
- Volume-mounted script sources
- Development-specific environment variables
- Custom deployment with hostPath volumes

**Usage**:
```bash
# Modify the hostPath to point to your local script directory
kubectl apply -f local-development.yaml
```

## Testing the Examples

### Prerequisites
1. A Kubernetes cluster with nuop manager deployed
2. The nuop CRDs installed (`kubectl apply -f operator/chart/crds/nuop.yaml`)

### Testing ConfigMap Replication
```bash
# Apply the config-replicator example
kubectl apply -f config-replicator.yaml

# The example includes a test ConfigMap - check if it gets replicated
kubectl get configmaps -A -l app.kubernetes.io/replicated-by

# Check operator logs
kubectl logs -l app.kubernetes.io/name=nuop
```

### Testing Secret Cloning
```bash
# Apply the secret-cloner example  
kubectl apply -f secret-cloner.yaml

# The example includes a test Secret - check if it gets cloned
kubectl get secrets -A -l app.kubernetes.io/replicated-by

# Check operator logs
kubectl logs -l app.kubernetes.io/name=nuop
```

## Key Concepts Demonstrated

### Manager + Managed Mode
All examples use the Manager + Managed deployment mode where:
- A manager watches NuOperator custom resources
- The manager creates managed operator deployments 
- Scripts are fetched from the nuop repository

### Real Script Integration
Examples use actual working scripts from `operator/scripts/`:
- `config-replicator` - Replicates ConfigMaps to target namespaces
- `secret-cloner` - Clones Secrets to target namespaces

### Proper RBAC
Working examples include correct RBAC configurations that grant scripts only the permissions they need.

### Label-Based Selection
Scripts watch for resources with specific labels:
- `app.kubernetes.io/replicate: "yes"` - Marks resources for replication/cloning

## Customization

To adapt these examples:

1. **Change target resources**: Modify `labelSelectors` in mappings
2. **Adjust permissions**: Update RBAC rules based on script requirements  
3. **Use custom scripts**: Change `sources[].location` to your script repository
4. **Configure namespaces**: Set target namespaces in resource annotations

## External Resources

- [NuOperator CRD Reference](../api/CRD.md)
- [Script Development Guide](../SCRIPT-DEVELOPMENT.md)
- [Deployment Guide](../DEPLOYMENT.md)