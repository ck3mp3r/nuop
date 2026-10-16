# NuOperator CRD Reference

The `NuOperator` Custom Resource Definition (CRD) is used in Manager + Managed mode to define script-based Kubernetes operators. This reference covers all fields and their usage.

## Overview

The NuOperator CRD allows you to declaratively define Kubernetes operators that:
- Fetch Nushell scripts from various sources (Git repositories, container images)
- Map scripts to specific Kubernetes resources based on selectors
- Inject environment variables and configuration
- Run in isolated deployments managed by the nuop manager

## Resource Structure

```yaml
apiVersion: kemper.buzz/v1alpha1
kind: NuOperator
metadata:
  name: example-operator
  namespace: default
spec:
  # Operator configuration
  image: ghcr.io/ck3mp3r/nuop:latest
  serviceAccountName: nuop-operator

  # Script sources
  sources: []

  # Resource mappings
  mappings: []

  # Environment variables
  env: []

status: {}
```

## Specification Fields

### `spec.image` (string, optional)

Container image for the managed operator deployment.

**Default**: Uses the same image as the manager
**Example**: `ghcr.io/ck3mp3r/nuop:latest`

```yaml
spec:
  image: ghcr.io/ck3mp3r/nuop:latest
```

### `spec.serviceAccountName` (string, optional)

Service account for the managed operator pod. Must have appropriate RBAC permissions for the resources being managed.

**Default**: `default`
**Example**: `nuop-operator`

```yaml
spec:
  serviceAccountName: nuop-operator
```

### `spec.sources` (array, required)

Defines where to fetch operator content from. Each source represents a location containing either operator scripts or CRD manifests.

#### Source Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `location` | string | Yes | URL or path to fetch content from |
| `path` | string | Yes | Operator identity; content is fetched to `/sources/<path>/<scripts|crds>/` |
| `source_type` | string | No | `scripts` (default) or `crds` |
| `credentials` | object | No | Authentication credentials |

#### `source_type` (string, optional)

Selects what the source provides and where `init-sources` fetches it:

| Value | Destination | Description |
|-------|-------------|-------------|
| `scripts` | `/sources/<path>/scripts/` | Nushell operator scripts (`mod.nu`). Default when omitted. |
| `crds` | `/sources/<path>/crds/` | CRD manifests, auto-installed by `install-crds` before the operator starts. |

Sources that share the same `path` merge into the same operator directory. A common pattern is one `scripts` source and one `crds` source with the same `path` — the scripts come from the Nushell operator's repo and the CRDs come from the original operator's repo.

```yaml
spec:
  sources:
    # Nushell operator scripts — required
    - location: https://github.com/myorg/nushell-kafka-mimic.git?ref=main&dir=operators/kafka
      path: kafka
      source_type: scripts

    # CRDs from the original operator's repo — optional
    - location: https://github.com/strimzi/strimzi-kafka-operator.git?ref=main&dir=install/cluster-operator/src/main/resources
      path: kafka
      source_type: crds
```

Both entries use `path: kafka`, so the scripts land in `/sources/kafka/scripts/` and the CRDs in `/sources/kafka/crds/`.

#### Supported Location Formats

- **Git repositories**: `https://github.com/user/repo.git?ref=main&dir=scripts`
- **Local paths**: `/path/to/scripts` (when volume mounted)
- **Query parameters**:
  - `ref`: Git branch, tag, or commit (default: `main`)
  - `dir`: Subdirectory within repository (default: root)

#### Credentials

| Field | Type | Description |
|-------|------|-------------|
| `username` | SecretKeySelector | Reference to secret containing username |
| `password` | SecretKeySelector | Reference to secret containing password |
| `token` | SecretKeySelector | Reference to secret containing token |

**Examples**:

```yaml
spec:
  sources:
    # Public Git repository
    - location: https://github.com/your-org/operator-scripts.git?ref=v1.0.0&dir=operators
      path: scripts/public

    # Private repository with authentication
    - location: https://github.com/company/internal-operators.git
      path: scripts/private
      credentials:
        username:
          name: git-credentials
          key: username
        password:
          name: git-credentials
          key: token

    # Local volume mount
    - location: /opt/scripts
      path: scripts/local
```

### `spec.mappings` (array, required)

Defines which Kubernetes resources should trigger which scripts. Each mapping connects a resource type to a specific script.

#### Mapping Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | No | Name of the script to execute (default: "") |
| `kind` | string | Yes | Kubernetes resource kind |
| `version` | string | Yes | API version of the resource |
| `group` | string | No | API group (for custom resources, default: "") |
| `labelSelectors` | object | No | Label-based resource filtering |
| `fieldSelectors` | object | No | Field-based resource filtering |
| `requeue_after_noop` | integer | No | Requeue interval when no changes |
| `requeue_after_change` | integer | No | Requeue interval after changes made |

#### Selector Examples

**Label Selector**:
```yaml
labelSelectors:
  app.kubernetes.io/replicate: "yes"
  environment: production
```

**Field Selector**:
```yaml
fieldSelectors:
  metadata.namespace: "monitoring"
  spec.type: "ClusterIP"
```

**Complete Mapping Example**:
```yaml
spec:
  mappings:
    # Replicate ConfigMaps across namespaces
    - name: config-replicator
      kind: ConfigMap
      version: v1
      labelSelectors:
        app.kubernetes.io/replicate: "yes"
      requeue_after_noop: 300

    # Manage custom resources
    - name: backup-controller
      kind: Backup
      version: v1
      group: backup.example.com
      fieldSelectors:
        spec.schedule: "0 2 * * *"
```

### `spec.env` (array, optional)

Environment variables injected into the managed operator deployment. Supports static values and references to ConfigMaps, Secrets, and pod fields.

#### Environment Variable Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Environment variable name |
| `value` | string | No | Static value |
| `valueFrom` | object | No | Dynamic value source |

#### Value Sources

**ConfigMap Reference**:
```yaml
env:
  - name: LOG_LEVEL
    valueFrom:
      configMapKeyRef:
        name: operator-config
        key: log-level
        optional: true
```

**Secret Reference**:
```yaml
env:
  - name: API_TOKEN
    valueFrom:
      secretKeyRef:
        name: api-credentials
        key: token
```

**Field Reference**:
```yaml
env:
  - name: POD_NAMESPACE
    valueFrom:
      fieldRef:
        fieldPath: metadata.namespace
```

**Resource Reference**:
```yaml
env:
  - name: MEMORY_LIMIT
    valueFrom:
      resourceFieldRef:
        resource: limits.memory
```

**Complete Environment Example**:
```yaml
spec:
  env:
    # Static configuration
    - name: LOG_LEVEL
      value: debug

    # From ConfigMap
    - name: REQUEUE_INTERVAL
      valueFrom:
        configMapKeyRef:
          name: operator-config
          key: requeue-seconds

    # From Secret
    - name: GITHUB_TOKEN
      valueFrom:
        secretKeyRef:
          name: git-credentials
          key: token

    # Pod metadata
    - name: OPERATOR_NAMESPACE
      valueFrom:
        fieldRef:
          fieldPath: metadata.namespace
```

## Complete Example

```yaml
apiVersion: kemper.buzz/v1alpha1
kind: NuOperator
metadata:
  name: multi-resource-operator
  namespace: operators
spec:
  image: ghcr.io/ck3mp3r/nuop:latest
  serviceAccountName: nuop-operator

  sources:
    - location: https://github.com/company/k8s-operators.git?ref=v2.1.0&dir=scripts
      path: scripts/core
      credentials:
        username:
          name: git-credentials
          key: username
        password:
          name: git-credentials
          key: token

    - location: /opt/custom-scripts
      path: scripts/local

  mappings:
    # ConfigMap replication
    - name: config-replicator
      kind: ConfigMap
      version: v1
      labelSelectors:
        replicate: "true"
      requeue_after_noop: 600

    # Secret management
    - name: secret-rotator
      kind: Secret
      version: v1
      labelSelectors:
        rotate: "enabled"
      requeue_after_noop: 3600

    # Custom resource handling
    - name: backup-controller
      kind: DatabaseBackup
      version: v1alpha1
      group: database.company.com
      fieldSelectors:
        spec.enabled: "true"

  env:
    - name: LOG_LEVEL
      value: info

    - name: SLACK_WEBHOOK
      valueFrom:
        secretKeyRef:
          name: notification-config
          key: slack-webhook

    - name: BACKUP_BUCKET
      valueFrom:
        configMapKeyRef:
          name: backup-config
          key: s3-bucket

    - name: OPERATOR_NAMESPACE
      valueFrom:
        fieldRef:
          fieldPath: metadata.namespace
```

## Status Field

The `status` field is managed by the operator and reflects the current state of the managed deployment. While the specific status schema is extensible, it typically includes:

- Deployment status and readiness
- Error conditions and messages
- Last reconciliation timestamps
- Active script and source information

## Validation and Constraints

### Required Fields
- `spec.sources`: At least one source must be defined
- `spec.mappings`: At least one mapping must be defined
- Each mapping must specify `kind` and `version`. `name` is optional (defaults to empty string).
- Each source must have `location` and `path`.

### Field Validation
- Selector maps use string keys and values only
- Environment variable names must be valid C identifiers

### Resource Limits
- No explicit limits on number of sources, mappings, or environment variables
- Practical limits depend on Kubernetes etcd storage and operator performance

## Best Practices

### Security
- Use Secrets for sensitive environment variables
- Limit ServiceAccount permissions to required resources only
- Use specific image tags rather than `latest`
- Consider network policies for Git access

### Performance
- Use specific selectors to minimize watched resources
- Set appropriate requeue intervals based on reconciliation needs
- Avoid overly broad label selectors
- Monitor operator resource usage

### Maintainability
- Use descriptive names for sources and scripts
- Document custom selectors and their purpose
- Version your script sources with Git tags
- Group related mappings in the same NuOperator

## Related Documentation

- **[Script Development Guide](../SCRIPT-DEVELOPMENT.md)** - How to write operator scripts
- **[Examples](../examples/README.md)** - Complete NuOperator configurations
- **[Deployment Guide](../DEPLOYMENT.md)** - Production deployment practices
