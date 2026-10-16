# Secret Cloner

The Secret Cloner automatically replicates Secrets across namespaces with the same targeting system as the Config Replicator. It is a ready-to-use Nushell operator script for the Nushell Operator (nuop).

## Required Labels

- `app.kubernetes.io/replicate: "yes"` - Marks the Secret for replication

## Optional Annotations

- `app.kubernetes.io/target-namespaces: "namespace1,namespace2,namespace3"` - Comma-separated list of target namespaces
- `app.kubernetes.io/target-method: "include|exclude"` - Whether to include or exclude the specified namespaces (default: `include`)

## Behavior

- **Default**: If no target namespaces are specified, replicates to all namespaces except the source
- **Include Mode**: Replicates only to the specified namespaces in `target-namespaces`
- **Exclude Mode**: Replicates to all namespaces except those specified in `target-namespaces`
- **Automatic Cleanup**: When the source Secret is deleted, all replicas are automatically removed
- **Change Detection**: Only updates replicas when the source Secret data, type, or relevant labels change
- **Security**: Maintains original Secret type and data encoding

## Example Usage

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

## Replica Identification

The operator adds management labels to replicated resources:

- `app.kubernetes.io/managed-by: "github.com-ck3mp3r-nuop-sec-clnr"`
- `app.kubernetes.io/replicated-by: "github.com-ck3mp3r-nuop-sec-clnr"`

These labels identify which resources are managed by the operator and prevent infinite replication loops.

## Finalizers

The script uses a finalizer to ensure proper cleanup when source resources are deleted:

- `github.com/ck3mp3r-nuop-sec-clnr-finalizer`

## Requeue Behavior

The operator requeues after 60 seconds when no changes are detected, ensuring periodic reconciliation to catch any drift.

## Script Implementation

The `mod.nu` script follows the common operator pattern:

1. **Configuration**: The `main config` function returns operator metadata including the script name, target Kubernetes resource kind, label selectors, finalizer identifier, and requeue timing.

2. **Reconciliation**: The `main reconcile` function handles create/update events. It fetches the source resource, determines target namespaces based on annotations, creates or updates replicas as needed, and returns exit code 0 (no change) or 2 (changed).

3. **Finalization**: The `main finalize` function handles deletion events. It cleans up all replicated resources and removes finalizers to allow source resource deletion.

### Execution Model

Scripts are executed via the Nushell interpreter:

```bash
nu docs/examples/secret-cloner/scripts/mod.nu config      # Get configuration
nu docs/examples/secret-cloner/scripts/mod.nu reconcile   # Handle resource changes
nu docs/examples/secret-cloner/scripts/mod.nu finalize    # Handle resource deletion
```

In containers, scripts are at `/sources/<op-name>/scripts/mod.nu`.

## Usage with Nushell Operator

This script is designed to work with the Nushell Operator in Standard Mode. In the container image it is copied to `/sources/<op-name>/scripts/` and discovered by the operator scanning `/sources/`.

To use this script:

1. Include it in your custom operator image
2. Deploy the image to your Kubernetes cluster
3. Apply Secrets with the appropriate labels to trigger replication

The operator will automatically detect and manage resources that match its label selectors.
