# Standard Mode Deployment

Deploy self-contained operators with scripts bundled into a custom container image. This is the recommended mode for most production deployments.

## 1. Create Custom Operator Image

Bundle your scripts into a custom container image:

```dockerfile
# Dockerfile
FROM ghcr.io/ck3mp3r/nuop:latest

# Copy your operator scripts
COPY scripts/ /sources/

# Scripts will be automatically discovered and registered
```

Build and push your image:

```bash
docker build -t your-registry/your-operator:v1.0.0 .
docker push your-registry/your-operator:v1.0.0
```

## 2. Configure RBAC for Script Permissions

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

## 3. Deploy Operator

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
        - name: LOG_LEVEL
          value: info
        - name: NUOP_SOURCES_PATH
          value: /sources
        resources:
          requests:
            memory: "64Mi"
            cpu: "250m"
          limits:
            memory: "128Mi"
            cpu: "500m"
```

## Directory Layout

All operator content lives in a single `/sources/` tree, one directory per operator:

```
/sources/
└── <op-name>/
    ├── scripts/                  # Operator scripts, discovered by find_scripts
    │   └── mod.nu
    └── crds/                     # CRDs auto-installed from here
        ├── myresource-crd.yaml
        └── another-crd.yml
```

- `scripts/` holds the operator's `mod.nu`. `find_scripts` discovers `mod.nu` recursively anywhere under `/sources/`.
- `crds/` holds CRD manifests. `install-crds` scans every `crds/` directory under `/sources/` for `.yaml` and `.yml` files.

The `<op-name>` segment is the operator identity (for example `kafka` or `greeting-operator`). It is arbitrary — both scans are recursive.

## Startup Sequence

In Standard mode, CRDs are baked into the image:

1. **Container starts** with `NUOP_MODE=standard` (default)
2. **`install-crds` runs** before the operator starts, if any `crds/` directory under `/sources/` contains YAML files
3. **CRDs are installed/updated** using `kubectl apply`
4. **Each CRD is waited on** until `condition=established`
5. **Operator starts** and begins watching resources

## Baking CRDs into the Image

**1. Create your operator script** (`scripts/my-operator/mod.nu`):

```nushell
def "main config" [] {
    {
        name: "my-operator",
        kind: "MyCustomResource",
        group: "example.com",
        version: "v1"
    } | to yaml
}

def "main reconcile" [] {
    let resource = $in | from yaml
    print $"Processing ($resource.metadata.name)"
    # Your reconciliation logic here
    exit 2
}

def "main finalize" [] {
    let resource = $in | from yaml
    print $"Cleaning up ($resource.metadata.name)"
    exit 0
}
```

**2. Create your CRD** (`crds/mycustomresource-crd.yaml`):

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: mycustomresources.example.com
spec:
  group: example.com
  names:
    kind: MyCustomResource
    plural: mycustomresources
    singular: mycustomresource
  scope: Namespaced
  versions:
    - name: v1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              properties:
                field1:
                  type: string
```

**3. Create your Dockerfile**:

```dockerfile
FROM ghcr.io/ck3mp3r/nuop:latest

# Copy operator scripts into the unified sources tree
COPY scripts/ /sources/my-operator/scripts/

# Copy CRD definitions
COPY crds/ /sources/my-operator/crds/

# CRDs will be automatically installed on startup
```

**4. Build and push your image**:

```bash
docker build -t your-registry/my-operator:v1.0.0 .
docker push your-registry/my-operator:v1.0.0
```

## RBAC with CRD Permissions

If your operator manages custom resources, its ServiceAccount needs CRD permissions:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: my-operator
rules:
  # CRD management (required for auto-installation)
  - apiGroups: ["apiextensions.k8s.io"]
    resources: ["customresourcedefinitions"]
    verbs: ["get", "list", "create", "update", "patch"]

  # Your custom resource management
  - apiGroups: ["example.com"]
    resources: ["mycustomresources"]
    verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
```

## Related Documentation

- [Deployment Overview](README.md)
- [Manager + Managed Mode](managed-mode.md)
- [Automatic CRD Installation](crd-auto-installation.md)
- [Script Development Guide](../scripting/README.md)
