# Operator Scripts

This directory contains ready-to-use Nushell operator scripts that implement common Kubernetes patterns. These scripts are designed to be used with the Nushell Operator (nuop) and demonstrate how to build controllers using Nushell.

## Script Structure

Each operator script is organized as a directory containing:

- **`mod.nu`**: Main entry point with the operator logic
- **Additional files**: Optional helper modules and utilities

### Directory Layout
```
scripts/
├── config-replicator/
│   └── mod.nu              # ConfigMap replication logic
└── secret-cloner/
    └── mod.nu              # Secret replication logic
```

## Config Replicator (`config-replicator/`)

The Config Replicator automatically replicates ConfigMaps across namespaces based on labels and annotations.

### Required Labels
- `app.kubernetes.io/replicate: "yes"` - Marks the ConfigMap for replication

### Optional Annotations
- `app.kubernetes.io/target-namespaces: "namespace1,namespace2,namespace3"` - Comma-separated list of target namespaces
- `app.kubernetes.io/target-method: "include|exclude"` - Whether to include or exclude the specified namespaces (default: `include`)

### Behavior
- **Default**: If no target namespaces are specified, replicates to all namespaces except the source
- **Include Mode**: Replicates only to the specified namespaces in `target-namespaces`
- **Exclude Mode**: Replicates to all namespaces except those specified in `target-namespaces`
- **Automatic Cleanup**: When the source ConfigMap is deleted, all replicas are automatically removed
- **Change Detection**: Only updates replicas when the source ConfigMap data or relevant labels change

### Example Usage

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: default
  labels:
    app.kubernetes.io/replicate: "yes"
  annotations:
    app.kubernetes.io/target-namespaces: "production,staging"
    app.kubernetes.io/target-method: "include"
data:
  config.yaml: |
    app:
      environment: production
      debug: false
```

This ConfigMap will be replicated to the `production` and `staging` namespaces only.

## Secret Cloner (`secret-cloner/`)

The Secret Cloner automatically replicates Secrets across namespaces with the same targeting system as the Config Replicator.

### Required Labels
- `app.kubernetes.io/replicate: "yes"` - Marks the Secret for replication

### Optional Annotations
- `app.kubernetes.io/target-namespaces: "namespace1,namespace2,namespace3"` - Comma-separated list of target namespaces
- `app.kubernetes.io/target-method: "include|exclude"` - Whether to include or exclude the specified namespaces (default: `include`)

### Behavior
- **Default**: If no target namespaces are specified, replicates to all namespaces except the source
- **Include Mode**: Replicates only to the specified namespaces in `target-namespaces`
- **Exclude Mode**: Replicates to all namespaces except those specified in `target-namespaces`
- **Automatic Cleanup**: When the source Secret is deleted, all replicas are automatically removed
- **Change Detection**: Only updates replicas when the source Secret data, type, or relevant labels change
- **Security**: Maintains original Secret type and data encoding

### Example Usage

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: database-credentials
  namespace: default
  labels:
    app.kubernetes.io/replicate: "yes"
  annotations:
    app.kubernetes.io/target-namespaces: "kube-system,monitoring"
    app.kubernetes.io/target-method: "exclude"
type: Opaque
data:
  username: YWRtaW4=  # base64 encoded
  password: MWYyZDFlMmU2N2Rm  # base64 encoded
```

This Secret will be replicated to all namespaces except `kube-system` and `monitoring`.

## Common Features

### Replica Identification

Both operators add management labels to replicated resources:
- `app.kubernetes.io/managed-by: "github.com-ck3mp3r-nuop-[operator-name]"`
- `app.kubernetes.io/replicated-by: "github.com-ck3mp3r-nuop-[operator-name]"`

These labels help identify which resources are managed by the operators and prevent infinite replication loops.

### Finalizers

Both scripts use finalizers to ensure proper cleanup when source resources are deleted:
- Config Replicator: `github.com/ck3mp3r-nuop-cfg-repktr-finalizer`
- Secret Cloner: `github.com/ck3mp3r-nuop-sec-clnr-finalizer`

### Requeue Behavior

Both operators are configured to requeue after 60 seconds when no changes are detected, ensuring periodic reconciliation to catch any drift.

## Script Implementation

Each operator script (`mod.nu`) follows a common pattern:

1. **Configuration**: The `main config` function returns operator metadata including:
   - Script name and target Kubernetes resource kind
   - Label selectors for filtering resources
   - Finalizer identifier
   - Requeue timing

2. **Reconciliation**: The `main reconcile` function handles create/update events:
   - Fetches the source resource
   - Determines target namespaces based on annotations
   - Creates or updates replicas as needed
   - Returns appropriate exit codes (0 = no change, 2 = changed)

3. **Finalization**: The `main finalize` function handles deletion events:
   - Cleans up all replicated resources
   - Removes finalizers to allow source resource deletion

### Execution Model

Scripts are executed via the Nushell interpreter:
```bash
nu scripts/config-replicator/mod.nu config      # Get configuration
nu scripts/config-replicator/mod.nu reconcile   # Handle resource changes
nu scripts/config-replicator/mod.nu finalize    # Handle resource deletion
```

## Usage with Nushell Operator

These scripts are designed to work with the Nushell Operator in Standard Mode, where they are bundled into container images and automatically discovered based on their metadata configuration.

To use these scripts:

1. Include them in your custom operator image
2. Deploy the image to your Kubernetes cluster
3. Apply ConfigMaps or Secrets with the appropriate labels to trigger replication

The operators will automatically detect and manage resources that match their label selectors.

