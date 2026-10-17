# NuOperator Examples

Working example configurations for the NuOperator CRD. Each operator is self-contained at the top level with its scripts, manifests, and RBAC. Generic deployment templates live in `templates/`.

## Standard Mode Templates

Scripts are bundled into a custom container image at build time. See the [Standard Mode deployment guide](../deployment/standard-mode.md).

| Template | Purpose |
|---------|---------|
| [`templates/standard-mode.yaml`](templates/standard-mode.yaml) | Standard mode deployment with scripts bundled in a container image |
| [`templates/custom-crd-operator.yaml`](templates/custom-crd-operator.yaml) | Standard mode with automatic CRD installation |

```bash
kubectl apply -f templates/standard-mode.yaml
kubectl apply -f templates/custom-crd-operator.yaml
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
kubectl apply -f templates/standard-mode.yaml
```

### custom-crd-operator.yaml

Shows Standard mode with automatic CRD installation.

**Features shown**:
- CRD baked into the image under `/sources/<op-name>/crds/`
- Automatic CRD installation on startup
- Custom resource reconciliation

```bash
kubectl apply -f templates/custom-crd-operator.yaml
```

## Managed Mode Templates

A manager watches `NuOperator` custom resources and provisions managed operator deployments. Scripts are fetched from git sources at runtime. See the [Manager + Managed Mode deployment guide](../deployment/managed-mode.md).

| Example | Purpose |
|---------|---------|
| [`templates/minimal.yaml`](templates/minimal.yaml) | Minimal `NuOperator` CR — required fields only |
| [`config-replicator/managed/nuoperator.yaml`](config-replicator/managed/nuoperator.yaml) | ConfigMap replication across namespaces |
| [`secret-cloner/managed/nuoperator.yaml`](secret-cloner/managed/nuoperator.yaml) | Secret cloning across namespaces |
| [`templates/local-development.yaml`](templates/local-development.yaml) | Local development with volume-mounted scripts |

```bash
kubectl apply -f templates/minimal.yaml
kubectl apply -f config-replicator/rbac.yaml
kubectl apply -f config-replicator/managed/nuoperator.yaml
kubectl apply -f secret-cloner/rbac.yaml
kubectl apply -f secret-cloner/managed/nuoperator.yaml
kubectl apply -f templates/local-development.yaml
```

### minimal.yaml

The simplest possible `NuOperator` configuration for learning the basics.

**Features shown**:
- Minimal required fields only
- No RBAC (uses the default service account)
- Basic mapping configuration

### config-replicator

Demonstrates ConfigMap replication across namespaces using the built-in `config-replicator` script. See [`config-replicator/`](config-replicator/) for the RBAC, standard-mode Deployment, and managed-mode manifests.

**Features shown**:
- Manager + Managed mode deployment
- Real script usage (`config-replicator` from the nuop repository)
- Proper RBAC configuration for ConfigMap operations
- Label-based resource selection
- Complete example with a test ConfigMap

### secret-cloner

Demonstrates Secret cloning across namespaces using the built-in `secret-cloner` script. See [`secret-cloner/`](secret-cloner/) for the RBAC, standard-mode Deployment, and managed-mode manifests.

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
kubectl apply -f templates/local-development.yaml
```

## Operator Examples

Complete operators that reconcile a custom resource, with manifests for both deployment modes.

### CockroachDB Dev Operator

[`cockroachdb-dev/`](cockroachdb-dev/) reconciles `CrdbCluster` resources (`crdb.cockroachlabs.com/v1beta1`) for local development. It is a lightweight alternative to the [CockroachDB operator](https://github.com/cockroachdb/helm-charts) and produces the same external interface, so Helm charts work the same in development and production.

| File | Purpose |
|------|---------|
| [`cockroachdb-dev/rbac.yaml`](cockroachdb-dev/rbac.yaml) | ServiceAccount, ClusterRole, and ClusterRoleBinding |
| [`cockroachdb-dev/docker/Dockerfile`](cockroachdb-dev/docker/Dockerfile) | Standard mode image with the script and CRD bundled |
| [`cockroachdb-dev/standard/deployment.yaml`](cockroachdb-dev/standard/deployment.yaml) | Standard mode Deployment |
| [`cockroachdb-dev/managed/nuoperator.yaml`](cockroachdb-dev/managed/nuoperator.yaml) | Managed mode `NuOperator` CR |
| [`cockroachdb-dev/crdb-cluster.yaml`](cockroachdb-dev/crdb-cluster.yaml) | Example `CrdbCluster` for local development |

**Standard mode**:

```bash
docker build -t cockroachdb-dev -f docs/examples/cockroachdb-dev/docker/Dockerfile docs/examples/cockroachdb-dev/
kubectl apply -f cockroachdb-dev/rbac.yaml
kubectl apply -f cockroachdb-dev/standard/deployment.yaml
kubectl apply -f cockroachdb-dev/crdb-cluster.yaml
```

**Managed mode**:

```bash
kubectl apply -f cockroachdb-dev/rbac.yaml
kubectl apply -f cockroachdb-dev/managed/nuoperator.yaml
kubectl apply -f cockroachdb-dev/crdb-cluster.yaml
```

See [`cockroachdb-dev/README.md`](cockroachdb-dev/README.md) for the full operator reference.

## Testing the Examples

### Prerequisites

1. A Kubernetes cluster with the nuop manager deployed
2. The nuop CRDs installed (`kubectl apply -f operator/chart/crds/nuop.yaml`)

### Testing ConfigMap Replication

```bash
# Apply the config-replicator example
kubectl apply -f config-replicator/rbac.yaml
kubectl apply -f config-replicator/managed/nuoperator.yaml
kubectl apply -f config-replicator/managed/example-configmap.yaml

# The example includes a test ConfigMap - check if it gets replicated
kubectl get configmaps -A -l app.kubernetes.io/replicated-by

# Check operator logs
kubectl logs -l app.kubernetes.io/name=nuop
```

### Testing Secret Cloning

```bash
# Apply the secret-cloner example
kubectl apply -f secret-cloner/rbac.yaml
kubectl apply -f secret-cloner/managed/nuoperator.yaml
kubectl apply -f secret-cloner/managed/example-secret.yaml

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

Examples use actual working scripts from `docs/examples/`:
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
- **Examples**: [`templates/standard-mode.yaml`](templates/standard-mode.yaml), [`cockroachdb-dev/standard/`](cockroachdb-dev/standard/), [`config-replicator/standard/`](config-replicator/standard/), [`secret-cloner/standard/`](secret-cloner/standard/)

### Manager + Managed Mode

- **Best for**: Dynamic operator provisioning, multi-tenant environments
- **How it works**: Manager watches NuOperator CRDs and creates managed operator deployments
- **Script source**: Fetched from Git repositories or other sources at runtime
- **Examples**: [`templates/minimal.yaml`](templates/minimal.yaml), [`cockroachdb-dev/managed/`](cockroachdb-dev/managed/), [`config-replicator/managed/`](config-replicator/managed/), [`secret-cloner/managed/`](secret-cloner/managed/)

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
