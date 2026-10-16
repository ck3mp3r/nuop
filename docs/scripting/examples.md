# Script Examples

See the [docs/examples/operators](../examples/operators/) directory for complete working examples:

- **config-replicator**: Replicates ConfigMaps across namespaces
- **secret-cloner**: Replicates Secrets with targeting controls

These examples demonstrate real-world patterns and can serve as templates for your own operators.

## Config Replicator (`docs/examples/operators/config-replicator/`)

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

## Secret Cloner (`docs/examples/operators/secret-cloner/`)

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

## Related Documentation

- [Script Development Tutorial](README.md)
- [Script API Reference](api-reference.md)
- [docs/examples/operators/](../examples/operators/)
