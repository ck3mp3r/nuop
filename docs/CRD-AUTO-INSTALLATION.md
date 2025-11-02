# Automatic CRD Installation

The Nushell Operator supports automatic installation of Custom Resource Definitions (CRDs) when starting in **Standard mode**. This feature simplifies deployment of operators that manage custom resources.

## Overview

When the operator starts in Standard mode, it automatically:

1. Checks for a `/crds` directory in the container
2. Discovers all `*.yaml` files in the directory
3. Installs or updates each CRD
4. Waits for CRDs to be established before starting controllers

This ensures your custom resources are ready before your operator scripts begin reconciliation.

## How It Works

### Directory Structure

Place your CRD definitions in the `/crds` directory of your operator container:

```
/
├── bin/
│   ├── operator
│   ├── entrypoint
│   └── install-crds
├── scripts/
│   └── my-operator/
│       └── mod.nu
└── crds/                          # CRDs auto-installed from here
    ├── myresource-crd.yaml
    └── another-crd.yaml
```

### Startup Sequence

1. **Container starts** with `NUOP_MODE=standard` (default)
2. **`install-crds` runs** before the operator starts
3. **CRDs are installed/updated** using `kubectl apply`
4. **CRDs are validated** to ensure they're established
5. **Operator starts** and begins watching resources

## Usage

### Building a Custom Operator with CRDs

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

# Copy operator scripts
COPY scripts/ /scripts/

# Copy CRD definitions
COPY crds/ /crds/

# CRDs will be automatically installed on startup
```

**4. Build and push your image**:

```bash
docker build -t your-registry/my-operator:v1.0.0 .
docker push your-registry/my-operator:v1.0.0
```

### Deploying Your Operator

**1. Create RBAC with CRD permissions**:

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

**2. Deploy your operator**:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-operator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: my-operator
  template:
    metadata:
      labels:
        app: my-operator
    spec:
      serviceAccountName: my-operator
      containers:
        - name: operator
          image: your-registry/my-operator:v1.0.0
          env:
            - name: NUOP_MODE
              value: standard  # Default mode
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CRDS_PATH` | `/crds` | Directory containing CRD YAML files |
| `NUOP_MODE` | `standard` | Operator mode (must be `standard` for auto-install) |

### Custom CRD Path

Override the default `/crds` path if needed:

```yaml
env:
  - name: CRDS_PATH
    value: /custom/path/to/crds
```

## Features

### Idempotent Installation

- CRDs that already exist are updated (if different)
- Safe to run multiple times
- Handles operator restarts gracefully

### Validation

- Validates CRD YAML format before applying
- Waits for CRDs to be "established" (ready to use)
- Provides clear error messages for invalid CRDs

### Multiple CRDs

- Supports any number of CRD files
- Processes all `*.yaml` files in `/crds` directory
- Can organize CRDs in subdirectories: `/crds/**/*.yaml`

### Error Handling

- Continues processing remaining CRDs if one fails
- Reports summary of successes and failures
- Exits with error code if any CRD installation fails

## Output Examples

### Successful Installation

```
🔍 Checking for CRDs to install...
📦 Found 2 CRD file(s) to process
📄 Processing: /crds/myresource-crd.yaml
➕ Installing new CRD: myresources.example.com
✅ CRD myresources.example.com installed successfully
⏳ Waiting for CRD myresources.example.com to be established...
✅ CRD myresources.example.com is ready
📄 Processing: /crds/another-crd.yaml
✅ CRD anotherresources.example.com already exists - updating...
✅ CRD anotherresources.example.com updated successfully

📊 CRD Installation Summary:
   ✅ Successful: 2
   ❌ Errors: 0
✅ All CRDs installed successfully
```

### No CRDs Found

```
🔍 Checking for CRDs to install...
ℹ️  No /crds directory found - skipping CRD installation
```

### Installation Error

```
🔍 Checking for CRDs to install...
📦 Found 1 CRD file(s) to process
📄 Processing: /crds/invalid-crd.yaml
❌ Error: Invalid CRD format in /crds/invalid-crd.yaml

📊 CRD Installation Summary:
   ✅ Successful: 0
   ❌ Errors: 1
⚠️  Some CRDs failed to install - operator may not work correctly
```

## Best Practices

### 1. CRD Versioning

Use explicit versions in your CRD definitions to ensure compatibility:

```yaml
apiVersion: apiextensions.k8s.io/v1  # Use v1, not v1beta1
```

### 2. Validation Schemas

Always include OpenAPI schemas for your custom resources:

```yaml
schema:
  openAPIV3Schema:
    type: object
    required: ["spec"]
    properties:
      spec:
        type: object
        # Define your fields here
```

### 3. RBAC Permissions

Grant minimal CRD permissions:

```yaml
# Sufficient for auto-installation
- apiGroups: ["apiextensions.k8s.io"]
  resources: ["customresourcedefinitions"]
  verbs: ["get", "list", "create", "update", "patch"]
```

### 4. Testing

Test your CRDs locally before building your operator:

```bash
# Validate CRD syntax
kubectl apply --dry-run=client -f crds/mycrd.yaml

# Test CRD installation
kubectl apply -f crds/mycrd.yaml
kubectl get crd mycustomresources.example.com

# Clean up
kubectl delete crd mycustomresources.example.com
```

### 5. Naming Conventions

Follow Kubernetes CRD naming conventions:

- **Metadata name**: `<plural>.<group>` (e.g., `databasebackups.backup.example.com`)
- **Group**: Use your domain (e.g., `backup.example.com`)
- **Kind**: PascalCase singular (e.g., `DatabaseBackup`)
- **Plural**: lowercase (e.g., `databasebackups`)

## Limitations

### Standard Mode Only

Automatic CRD installation only works in **Standard mode**:

- ✅ Standard mode: CRDs auto-installed
- ❌ Manager mode: CRDs must be installed separately
- ❌ Managed mode: CRDs must be installed separately

For Manager and Managed modes, install CRDs manually:

```bash
kubectl apply -f crds/
```

### Core Resources

CRD installation is only needed for **custom resources**:

- Core resources (ConfigMap, Secret, Pod, etc.) don't need CRDs
- Only use this feature if your operator manages custom resources

### Cluster Permissions

The operator ServiceAccount needs CRD management permissions. If your cluster has strict RBAC policies, you may need to coordinate with cluster admins.

## Troubleshooting

### "Command `install-crds` not found"

Ensure the `install-crds` script is in `/bin/` and executable:

```dockerfile
COPY ./docker/install-crds /bin/
RUN chmod +x /bin/install-crds
```

### "Permission denied" for CRD creation

Check that your ServiceAccount has CRD permissions:

```bash
kubectl auth can-i create customresourcedefinitions --as=system:serviceaccount:namespace:my-operator
```

### CRD not becoming "established"

If CRD installation hangs waiting for establishment:

1. Check CRD status: `kubectl get crd mycrd -o yaml`
2. Look for errors in status conditions
3. Verify the CRD schema is valid

### Invalid CRD format

Validate your CRD YAML:

```bash
# Check YAML syntax
yamllint crds/mycrd.yaml

# Validate against Kubernetes API
kubectl apply --dry-run=client -f crds/mycrd.yaml
```

## Examples

See complete examples in:

- [docs/examples/custom-crd-operator.yaml](examples/custom-crd-operator.yaml) - Full example with CRD, operator, and deployment

## Related Documentation

- [Script Development Guide](SCRIPT-DEVELOPMENT.md) - Writing operator scripts
- [Deployment Guide](DEPLOYMENT.md) - Deploying operators
- [Examples](examples/README.md) - More examples
