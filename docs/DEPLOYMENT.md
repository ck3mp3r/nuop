# Deployment Guide

This guide covers deploying the Nushell Operator in production environments, focusing on nuop-specific deployment modes and script permissions.

## Deployment Modes

### Standard Mode (Recommended)

Deploy self-contained operators with scripts bundled into container images.

**Advantages**:
- No external dependencies
- Simplified deployment
- Better security and isolation
- Faster startup times
- Easier version management

**Use Case**: Most production deployments where you have a specific set of operators to deploy.

### Manager + Managed Mode

Deploy a manager that dynamically provisions operators based on `NuOperator` custom resources.

**Advantages**:
- Dynamic operator provisioning
- Multi-tenant environments
- Centralized management
- Runtime script updates

**Use Case**: Platform teams managing multiple operators or multi-tenant environments.

## Standard Mode Deployment

### 1. Create Custom Operator Image

Bundle your scripts into a custom container image:

```dockerfile
# Dockerfile
FROM ghcr.io/ck3mp3r/nuop:latest

# Copy your operator scripts
COPY scripts/ /scripts/

# Scripts will be automatically discovered and registered
```

Build and push your image:

```bash
docker build -t your-registry/your-operator:v1.0.0 .
docker push your-registry/your-operator:v1.0.0
```

### 2. Configure RBAC for Script Permissions

Create RBAC resources that grant your scripts the necessary permissions to manage Kubernetes resources.

**Example: ConfigMap Replicator Script**

If your script manages ConfigMaps and needs to read Namespaces:

```yaml
# rbac.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: configmap-replicator
  namespace: your-namespace
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: configmap-replicator
rules:
# Permissions for ConfigMap operations
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
# Read access to namespaces (for replication targets)
- apiGroups: [""]
  resources: ["namespaces"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: configmap-replicator
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: configmap-replicator
subjects:
- kind: ServiceAccount
  name: configmap-replicator
  namespace: your-namespace
```

**Example: Secret Cloner Script**

If your script manages Secrets:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: secret-cloner
rules:
# Permissions for Secret operations
- apiGroups: [""]
  resources: ["secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
# Read access to namespaces
- apiGroups: [""]
  resources: ["namespaces"]
  verbs: ["get", "list", "watch"]
```

**Example: Multi-Resource Operator**

If your script manages multiple resource types:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: multi-resource-operator
rules:
# Core resources
- apiGroups: [""]
  resources: ["configmaps", "secrets", "services"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
# Apps resources
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch", "create", "update", "patch"]
# Networking resources
- apiGroups: ["networking.k8s.io"]
  resources: ["ingresses"]
  verbs: ["get", "list", "watch"]
```

### 3. Deploy Operator

Deploy your operator with the custom image:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: your-operator
  namespace: your-namespace
spec:
  replicas: 1
  selector:
    matchLabels:
      app: your-operator
  template:
    metadata:
      labels:
        app: your-operator
    spec:
      serviceAccountName: your-operator-sa
      containers:
      - name: operator
        image: your-registry/your-operator:v1.0.0
        env:
        - name: RUST_LOG
          value: info
        - name: NUOP_SCRIPT_PATH
          value: /scripts
        resources:
          requests:
            memory: "64Mi"
            cpu: "250m"
          limits:
            memory: "128Mi"
            cpu: "500m"
```

## Manager + Managed Mode Deployment

### 1. Deploy Manager

Deploy the nuop manager:

```yaml
# manager.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nuop-manager
  namespace: nuop-system
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: nuop-manager
  template:
    metadata:
      labels:
        app.kubernetes.io/name: nuop-manager
    spec:
      serviceAccountName: nuop-manager
      containers:
      - name: manager
        image: ghcr.io/ck3mp3r/nuop:latest
        env:
        - name: NUOP_MODE
          value: manager
        - name: RUST_LOG
          value: info
```

### 2. Install CRDs

Apply the NuOperator Custom Resource Definition:

```bash
# Apply CRDs from the repository
kubectl apply -f operator/chart/crds/nuop.yaml
```

### 3. Create NuOperator Resources

Define operators using NuOperator custom resources:

```yaml
# example-operator.yaml
apiVersion: kemper.buzz/v1alpha1
kind: NuOperator
metadata:
  name: example-operator
  namespace: default
spec:
  serviceAccountName: example-operator-sa
  sources:
    - location: https://github.com/your-org/operator-scripts.git?ref=main
      path: /scripts
  mappings:
    - name: configmap-controller
      group: ""
      version: v1
      kind: ConfigMap
      labelSelectors:
        app.kubernetes.io/managed-by: example-operator
```

## Script Permission Guidelines

### Principle of Least Privilege

Grant scripts only the permissions they need:

```yaml
# Good: Specific permissions for ConfigMap management
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]

# Avoid: Overly broad permissions
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]
```

### Common Permission Patterns

**Read-only monitoring scripts:**
```yaml
rules:
- apiGroups: [""]
  resources: ["pods", "services"]
  verbs: ["get", "list", "watch"]
```

**Resource replication scripts:**
```yaml
rules:
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["namespaces"]
  verbs: ["get", "list", "watch"]
```

**Application deployment scripts:**
```yaml
rules:
- apiGroups: ["apps"]
  resources: ["deployments"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["services"]
  verbs: ["get", "list", "watch", "create", "update", "patch"]
```

## Performance Tuning

### Requeue Intervals

Optimize requeue intervals in your script configuration:

```nushell
# In your script's config function
{
    name: "my-operator",
    # ... other config
    requeue_after_change: 10,   # Requeue 10 seconds after making changes
    requeue_after_noop: 300     # Requeue 5 minutes when no changes needed
}
```

**Guidelines:**
- Use shorter intervals (10-30s) after changes for quick reconciliation
- Use longer intervals (5-15 minutes) when no changes are needed
- Consider resource count and API server load

### Resource Selectors

Use specific selectors to reduce reconciliation load:

```nushell
{
    # Target specific resources with labels
    labelSelectors: {
        "app.kubernetes.io/managed-by": "my-operator"
        "environment": "production"
    },
    # Or use field selectors for namespace-specific operators
    fieldSelectors: {
        "metadata.namespace": "my-namespace"
    }
}
```

### Script Optimization

**Efficient resource checks:**
```nushell
# Check if changes are needed before applying
let existing = (kubectl get configmap $name -n $namespace -o yaml | complete)
if $existing.exit_code == 0 {
    let current = ($existing.stdout | from yaml)
    # Only apply if changes are needed
    if ($current.data != $desired.data) {
        # Apply changes
        exit 2
    } else {
        # No changes needed
        exit 0
    }
}
```

## Security Considerations

### RBAC Best Practices

- Grant scripts only the permissions they need for managed resources
- Use ServiceAccounts specific to each operator
- Avoid cluster-admin permissions unless absolutely necessary
- Regularly audit and review granted permissions

### Container Security

Run containers with minimal privileges:

```yaml
securityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 65534
  capabilities:
    drop:
    - ALL
```

## External Resources

For general Kubernetes deployment and security best practices, see:
- [Kubernetes Security Best Practices](https://kubernetes.io/docs/concepts/security/security-best-practices/)
- [Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)
- [RBAC Good Practices](https://kubernetes.io/docs/concepts/security/rbac-good-practices/)
- [Kubernetes Deployment Best Practices](https://kubernetes.io/docs/concepts/workloads/management/)