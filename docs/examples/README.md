# NuOperator Examples

Working example configurations for the NuOperator CRD, organized by deployment mode. Each example uses real scripts from the nuop repository.

## Standard Mode Examples

Scripts are bundled into a custom container image at build time. See the [Standard Mode deployment guide](../deployment/standard-mode.md).

| Example | Purpose |
|---------|---------|
| [`standard/standard-mode.yaml`](standard/standard-mode.yaml) | Standard mode deployment with scripts bundled in a container image |
| [`standard/custom-crd-operator.yaml`](standard/custom-crd-operator.yaml) | Standard mode with automatic CRD installation |

```bash
kubectl apply -f standard/standard-mode.yaml
kubectl apply -f standard/custom-crd-operator.yaml
```

### standard-mode.yaml

Shows Standard mode deployment with scripts bundled in container images.

**Features shown**:
- Standard mode configuration
- Custom image with bundled scripts
- Production deployment pattern

```bash
# Requires a custom image build first:
# docker build -t your-registry/nuop-with-scripts .
kubectl apply -f standard/standard-mode.yaml
```

### custom-crd-operator.yaml

Shows Standard mode with automatic CRD installation.

**Features shown**:
- CRD baked into the image under `/sources/<op-name>/crds/`
- Automatic CRD installation on startup
- Custom resource reconciliation

```bash
kubectl apply -f standard/custom-crd-operator.yaml
```

## Managed Mode Examples

A manager watches `NuOperator` custom resources and provisions managed operator deployments. Scripts are fetched from git sources at runtime. See the [Manager + Managed Mode deployment guide](../deployment/managed-mode.md).

| Example | Purpose |
|---------|---------|
| [`managed/minimal.yaml`](managed/minimal.yaml) | Minimal `NuOperator` CR — required fields only |
| [`managed/config-replicator.yaml`](managed/config-replicator.yaml) | ConfigMap replication across namespaces |
| [`managed/secret-cloner.yaml`](managed/secret-cloner.yaml) | Secret cloning across namespaces |
| [`managed/local-development.yaml`](managed/local-development.yaml) | Local development with volume-mounted scripts |

```bash
kubectl apply -f managed/minimal.yaml
kubectl apply -f managed/config-replicator.yaml
kubectl apply -f managed/secret-cloner.yaml
kubectl apply -f managed/local-development.yaml
```

### minimal.yaml

The simplest possible `NuOperator` configuration for learning the basics.

**Features shown**:
- Minimal required fields only
- No RBAC (uses the default service account)
- Basic mapping configuration

### config-replicator.yaml

Demonstrates ConfigMap replication across namespaces using the built-in `config-replicator` script.

**Features shown**:
- Manager + Managed mode deployment
- Real script usage (`config-replicator` from the nuop repository)
- Proper RBAC configuration for ConfigMap operations
- Label-based resource selection
- Complete example with a test ConfigMap

### secret-cloner.yaml

Demonstrates Secret cloning across namespaces using the built-in `secret-cloner` script.

**Features shown**:
- Secret management operations
- Cross-namespace secret replication
- Proper RBAC for Secret operations
- Complete example with a test Secret

### local-development.yaml

Shows how to develop and test scripts locally using volume mounts.

**Features shown**:
- Local script development workflow
- Volume-mounted script sources
- Development-specific environment variables
- Custom deployment with hostPath volumes

```bash
# Modify the hostPath to point to your local script directory
kubectl apply -f managed/local-development.yaml
```

## Operator Examples

Complete operators that reconcile a custom resource, with manifests for both deployment modes.

### CockroachDB Dev Operator

[`operators/cockroachdb-dev/`](operators/cockroachdb-dev/) reconciles `CrdbCluster` resources (`crdb.cockroachlabs.com/v1alpha1`) for local development. It is a lightweight alternative to the deprecated [CockroachDB Kubernetes operator](https://github.com/cockroachdb/cockroach-operator) and produces the same external interface, so Helm charts work the same in development and production.

| File | Purpose |
|------|---------|
| [`operators/cockroachdb-dev/rbac.yaml`](operators/cockroachdb-dev/rbac.yaml) | ServiceAccount, ClusterRole, and ClusterRoleBinding |
| [`operators/cockroachdb-dev/docker/Dockerfile`](operators/cockroachdb-dev/docker/Dockerfile) | Standard mode image with the script and CRD bundled |
| [`operators/cockroachdb-dev/standard/deployment.yaml`](operators/cockroachdb-dev/standard/deployment.yaml) | Standard mode Deployment |
| [`operators/cockroachdb-dev/managed/nuoperator.yaml`](operators/cockroachdb-dev/managed/nuoperator.yaml) | Managed mode `NuOperator` CR |
| [`operators/cockroachdb-dev/crdb-cluster.yaml`](operators/cockroachdb-dev/crdb-cluster.yaml) | Example `CrdbCluster` for local development |

**Standard mode**:

```bash
docker build -t cockroachdb-dev -f docs/examples/operators/cockroachdb-dev/docker/Dockerfile docs/examples/operators/cockroachdb-dev/
kubectl apply -f operators/cockroachdb-dev/rbac.yaml
kubectl apply -f operators/cockroachdb-dev/standard/deployment.yaml
kubectl apply -f operators/cockroachdb-dev/crdb-cluster.yaml
```

**Managed mode**:

```bash
kubectl apply -f operators/cockroachdb-dev/rbac.yaml
kubectl apply -f operators/cockroachdb-dev/managed/nuoperator.yaml
kubectl apply -f operators/cockroachdb-dev/crdb-cluster.yaml
```

See [`operators/cockroachdb-dev/README.md`](operators/cockroachdb-dev/README.md) for the full operator reference.

## Testing the Examples

### Prerequisites

1. A Kubernetes cluster with the nuop manager deployed
2. The nuop CRDs installed (`kubectl apply -f operator/chart/crds/nuop.yaml`)

### Testing ConfigMap Replication

```bash
# Apply the config-replicator example
kubectl apply -f managed/config-replicator.yaml

# The example includes a test ConfigMap - check if it gets replicated
kubectl get configmaps -A -l app.kubernetes.io/replicated-by

# Check operator logs
kubectl logs -l app.kubernetes.io/name=nuop
```

### Testing Secret Cloning

```bash
# Apply the secret-cloner example
kubectl apply -f managed/secret-cloner.yaml

# The example includes a test Secret - check if it gets cloned
kubectl get secrets -A -l app.kubernetes.io/replicated-by

# Check operator logs
kubectl logs -l app.kubernetes.io/name=nuop
```

## Key Concepts Demonstrated

### Manager + Managed Mode

The managed examples use the Manager + Managed deployment mode where:
- A manager watches NuOperator custom resources
- The manager creates managed operator deployments
- Scripts are fetched from the nuop repository

### Real Script Integration

Examples use actual working scripts from `docs/examples/operators/`:
- `config-replicator` - Replicates ConfigMaps to target namespaces
- `secret-cloner` - Clones Secrets to target namespaces

### Proper RBAC

Working examples include correct RBAC configurations that grant scripts only the permissions they need.

### Label-Based Selection

Scripts watch for resources with specific labels:
- `app.kubernetes.io/replicate: "yes"` - Marks resources for replication/cloning

## Deployment Modes

### Standard Mode

- **Best for**: Production deployments with fixed script sets
- **How it works**: Scripts are bundled into container images at build time
- **Script source**: Bundled in container at `/sources`
- **Examples**: [`standard/`](standard/)

### Manager + Managed Mode

- **Best for**: Dynamic operator provisioning, multi-tenant environments
- **How it works**: Manager watches NuOperator CRDs and creates managed operator deployments
- **Script source**: Fetched from Git repositories or other sources at runtime
- **Examples**: [`managed/`](managed/)

## Customization

To adapt these examples:

1. **Change target resources**: Modify `labelSelectors` in mappings
2. **Adjust permissions**: Update RBAC rules based on script requirements
3. **Use custom scripts**: Change `sources[].location` to your script repository
4. **Configure namespaces**: Set target namespaces in resource annotations

## External Resources

- [NuOperator CRD Reference](../api/nuoperator-crd.md)
- [Script Development Guide](../scripting/README.md)
- [Deployment Guide](../deployment/README.md)
- [Testing Guide](../development/testing.md)
