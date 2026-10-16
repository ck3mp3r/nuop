# Automatic CRD Installation

The Nushell Operator supports automatic installation of Custom Resource Definitions (CRDs) in both [Standard mode](standard-mode.md) and [Manager/Managed mode](managed-mode.md). This feature simplifies deployment of operators that manage custom resources.

## Overview

When the operator starts, it automatically:

1. Discovers CRD manifests in every `crds/` directory under `/sources/` (both `.yaml` and `.yml`)
2. Validates that each document has `kind: CustomResourceDefinition`
3. Installs or updates each CRD
4. Waits for each CRD to be `established` before starting controllers

This ensures your custom resources are ready before your operator scripts begin reconciliation.

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NUOP_SOURCES_PATH` | `/sources` | Root of the unified sources tree |
| `NUOP_MODE` | `standard` | Operator mode (`standard`, `init`, or `managed`) |

### Custom Sources Path

Override the default `/sources` root if needed:

```yaml
env:
  - name: NUOP_SOURCES_PATH
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

- [docs/examples/custom-crd-operator.yaml](../examples/custom-crd-operator.yaml) - Full example with CRD, operator, and deployment

## Related Documentation

- [Deployment Overview](README.md)
- [Standard Mode](standard-mode.md)
- [Manager + Managed Mode](managed-mode.md)
- [Script Development Guide](../SCRIPT-DEVELOPMENT.md)
- [Examples](../examples/README.md)
