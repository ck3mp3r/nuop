# Automatic CRD Installation

The Nushell Operator supports automatic installation of Custom Resource Definitions (CRDs) in both **Standard mode** and **Manager/Managed mode**. This feature simplifies deployment of operators that manage custom resources.

## Overview

When the operator starts, it automatically:

1. Discovers CRD manifests in every `crds/` directory under `/sources/` (both `.yaml` and `.yml`)
2. Validates that each document has `kind: CustomResourceDefinition`
3. Installs or updates each CRD
4. Waits for each CRD to be `established` before starting controllers

This ensures your custom resources are ready before your operator scripts begin reconciliation.

## How It Works

### Directory Structure

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

### Startup Sequence

**Standard mode** — CRDs are baked into the image:

1. **Container starts** with `NUOP_MODE=standard` (default)
2. **`install-crds` runs** before the operator starts, if any `crds/` directory under `/sources/` contains YAML files
3. **CRDs are installed/updated** using `kubectl apply`
4. **Each CRD is waited on** until `condition=established`
5. **Operator starts** and begins watching resources

**Manager/Managed mode** — CRDs are fetched from a source:

1. **Init container starts** with `NUOP_MODE=init`
2. **`init-sources` fetches** each `spec.sources` entry into `/sources/<path>/scripts/` or `/sources/<path>/crds/`, according to its `source_type`
3. **`install-crds` runs** in the init container if any CRDs were fetched
4. **Main container starts** the operator, which loads scripts from `/sources/`

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

### Manager/Managed Mode: Sourcing CRDs from a Git Repository

In Manager/Managed mode there is no Dockerfile to bake CRDs into. Instead, declare a second source with `source_type: crds` on the `NuOperator` CR. The manager's init container fetches it into `/sources/<path>/crds/`, and `install-crds` installs it before the operator starts.

```yaml
apiVersion: kemper.buzz/v1alpha1
kind: NuOperator
metadata:
  name: lightweight-kafka-operator
  namespace: operators
spec:
  image: ghcr.io/ck3mp3r/nuop:latest
  serviceAccountName: kafka-mimic-operator

  sources:
    # Nushell operator scripts (the lightweight mimic) — required
    - location: https://github.com/myorg/nushell-kafka-mimic.git?ref=main&dir=operators/kafka
      path: kafka
      source_type: scripts

    # CRDs from the original operator's repo — optional
    - location: https://github.com/strimzi/strimzi-kafka-operator.git?ref=main&dir=install/cluster-operator/src/main/resources
      path: kafka
      source_type: crds

  mappings:
    - name: kafka-controller
      kind: Kafka
      version: v1beta2
      group: kafka.strimzi.io
      requeue_after_noop: 300
      requeue_after_change: 30

  env:
    - name: LOG_LEVEL
      value: info
```

Both sources share `path: kafka`, so the scripts land in `/sources/kafka/scripts/` and the CRDs in `/sources/kafka/crds/`. The `source_type` field defaults to `scripts` when omitted.

The managed instance's ServiceAccount (named by `spec.serviceAccountName`) must carry the CRD permissions shown in [Deploying Your Operator](#deploying-your-operator).

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
| `SOURCES_PATH` | `/sources` | Root of the unified sources tree |
| `NUOP_MODE` | `standard` | Operator mode (`standard`, `init`, or `managed`) |

### Custom Sources Path

Override the default `/sources` root if needed:

```yaml
env:
  - name: SOURCES_PATH
    value: /custom/sources
```

## Features

### Idempotent Installation

- CRDs that already exist are updated (if different)
- Safe to run multiple times
- Handles operator restarts gracefully

### Validation

- Checks that each YAML document has `kind: CustomResourceDefinition` before applying; documents with any other kind are skipped with a warning
- Handles multi-document YAML files — every `CustomResourceDefinition` document in the file is applied
- Discovers both `.yaml` and `.yml` files
- Waits for each CRD to be `established` (ready to use) after applying, for both new and updated CRDs
- Provides clear error messages for unreadable or invalid CRDs

### Multiple CRDs

- Supports any number of CRD files
- Processes every `.yaml` and `.yml` file under any `crds/` directory in the sources tree
- Can organize CRDs in subdirectories

### Error Handling

- Continues processing remaining CRDs if one fails
- Reports summary of successes and failures
- Exits with error code if any CRD installation fails

## Output Examples

### Successful Installation

```
🔍 Checking for CRDs to install...
📦 Found 2 CRD file(s) to process
📄 Processing: /sources/my-operator/crds/myresource-crd.yaml
➕ Installing new CRD: myresources.example.com
✅ CRD myresources.example.com applied successfully
⏳ Waiting for CRD myresources.example.com to be established...
✅ CRD myresources.example.com is ready
📄 Processing: /sources/my-operator/crds/another-crd.yml
✅ CRD anotherresources.example.com already exists - updating...
✅ CRD anotherresources.example.com applied successfully
⏳ Waiting for CRD anotherresources.example.com to be established...
✅ CRD anotherresources.example.com is ready

📊 CRD Installation Summary:
   ✅ Successful: 2
   ❌ Errors: 0
✅ All CRDs installed successfully
```

### No CRDs Found

```
🔍 Checking for CRDs to install...
ℹ️  No CRD files found under /sources/**/crds/
```

### Skipped Document

A document whose `kind` is not `CustomResourceDefinition` is skipped:

```
📄 Processing: /sources/my-operator/crds/mixed.yaml
⚠️  Skipping document in /sources/my-operator/crds/mixed.yaml: kind is ConfigMap, not CustomResourceDefinition
```

### Installation Error

```
🔍 Checking for CRDs to install...
📦 Found 1 CRD file(s) to process
📄 Processing: /sources/my-operator/crds/invalid-crd.yaml
❌ Error: Could not read /sources/my-operator/crds/invalid-crd.yaml

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

### CRD Permissions Required

Automatic CRD installation works in both Standard mode and Manager/Managed mode, but the operator's ServiceAccount must be granted CRD permissions:

- ✅ Standard mode: CRDs baked into the image are auto-installed
- ✅ Manager/Managed mode: CRDs declared with `source_type: crds` are auto-installed
- ❌ Without CRD permissions: installation fails and the container exits non-zero

For Manager/Managed mode, the ServiceAccount named by `spec.serviceAccountName` must carry the CRD rule. The Helm chart gates that rule behind `rbac.crdAutoInstall` (default `false`).

### Core Resources

CRD installation is only needed for **custom resources**:

- Core resources (ConfigMap, Secret, Pod, etc.) don't need CRDs
- Only use this feature if your operator manages custom resources

### Cluster Permissions

The operator ServiceAccount needs CRD management permissions. If your cluster has strict RBAC policies, you may need to coordinate with cluster admins.

## Troubleshooting

### "Command `install-crds` not found"

Ensure the `install-crds` script is in `/bin/`:

```dockerfile
COPY ./docker/install-crds /bin/
```

The script ships with the executable bit set, so no `chmod` is required.

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
