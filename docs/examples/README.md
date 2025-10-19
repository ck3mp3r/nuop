# NuOperator Examples

This directory contains example configurations for the NuOperator CRD. These examples have been tested against nuop v0.2.0.

## Current Status & Limitations

**⚠️ Important**: Based on testing with nuop v0.2.0, there are some current limitations:

### Manager + Managed Mode
- ❌ **Current Issue**: Init container has Nushell compatibility issues (uses `--optional` flag that doesn't exist in current Nushell)
- 🔧 **Workaround**: Needs updated init container script
- ✅ **Manager works**: Creates deployments and ConfigMaps correctly
- 📝 **Use case**: Best for when the init container issue is resolved

### Standard Mode  
- ✅ **Works**: When scripts are bundled in container image
- ❌ **Limitation**: Requires custom image build with scripts included
- 📝 **Use case**: Production deployments with pre-built images

## Available Examples

### 1. minimal.yaml ✅ TESTED
**Purpose**: Simplest possible NuOperator configuration that validates successfully.

**Status**: ✅ Creates NuOperator resource successfully, but requires custom image with bundled scripts to function.

```bash
kubectl apply -f minimal.yaml
kubectl get nuoperators  # Shows resource created
```

### 2. config-replicator.yaml ⚠️ PARTIAL
**Purpose**: Demonstrates ConfigMap replication (Manager+Managed mode).

**Status**: ⚠️ Manager creates managed deployment, but init container fails due to Nushell compatibility.

**What works**:
- ✅ NuOperator resource creation
- ✅ Manager creates deployment and ConfigMaps
- ✅ RBAC configuration is correct

**What doesn't work**:
- ❌ Init container crashes (Nushell `--optional` flag issue)

```bash
kubectl apply -f config-replicator.yaml
kubectl get nuoperators,deployments,configmaps  # Shows resources created
kubectl logs deployment/config-replicator-nuop -c init-container  # Shows error
```

### 3. secret-cloner.yaml ⚠️ PARTIAL
**Purpose**: Demonstrates Secret cloning (Manager+Managed mode).

**Status**: ⚠️ Same issues as config-replicator due to shared init container.

### 4. local-development.yaml 📝 REFERENCE
**Purpose**: Shows volume-mounted local script development setup.

**Status**: 📝 Reference example for development workflow (requires local script directory).

### 5. standard-mode.yaml 📝 REFERENCE
**Purpose**: Shows Standard mode deployment with bundled scripts.

**Status**: 📝 Reference example (requires custom image build).

## Tested Functionality

### ✅ What Works
1. **Manager Deployment**: nuop manager starts and runs correctly
2. **CRD Validation**: All NuOperator resources validate and create successfully  
3. **Resource Creation**: Manager creates deployments, ConfigMaps, and RBAC correctly
4. **Script Syntax**: Local script testing works (config-replicator and secret-cloner scripts are valid)

### ❌ Current Issues
1. **Init Container**: Nushell compatibility issue with `--optional` flag
2. **Standard Mode**: Requires custom image with scripts bundled

### 🔧 Workarounds
1. **For Manager+Managed**: Wait for updated nuop image with fixed init container
2. **For Standard Mode**: Build custom image with scripts:
   ```dockerfile
   FROM ghcr.io/ck3mp3r/nuop:latest
   COPY scripts/ /scripts/
   ```

## Testing the Examples

### Prerequisites
```bash
# Start kind cluster
kind create cluster --name nuop --config kind/kind-cluster.yaml

# Install CRDs
kubectl apply -f operator/chart/crds/nuop.yaml

# Deploy manager (optional, for Manager+Managed mode)
kubectl apply -f test-deployment.yaml  # From project root
```

### Test Commands
```bash
# Test example validation
kubectl --dry-run=client apply -f minimal.yaml

# Apply examples
kubectl apply -f minimal.yaml
kubectl apply -f config-replicator.yaml

# Check created resources
kubectl get nuoperators,deployments,configmaps,pods

# Check logs
kubectl logs -n nuop-system deployment/nuop-manager  # Manager logs
kubectl logs deployment/config-replicator-nuop -c init-container  # Init container (will show error)
```

## Local Script Testing

You can test the scripts locally while waiting for container issues to be resolved:

```bash
# Test script configuration
echo '{}' | nu operator/scripts/config-replicator/mod.nu config

# Test with sample ConfigMap (create test-configmap.yaml first)
cat test-configmap.yaml | nu operator/scripts/config-replicator/mod.nu reconcile
```

## Next Steps

For fully working examples, the following needs to be addressed:

1. **Update init container script** in nuop image to use current Nushell syntax
2. **Provide pre-built images** with example scripts bundled for Standard mode
3. **Update examples** once container issues are resolved

## External Resources

- [NuOperator CRD Reference](../api/CRD.md)
- [Script Development Guide](../SCRIPT-DEVELOPMENT.md) 
- [Deployment Guide](../DEPLOYMENT.md)
- [Testing Guide](../TESTING.md)