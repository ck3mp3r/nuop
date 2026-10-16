# Manager + Managed Mode Deployment

Deploy a manager that dynamically provisions operators based on `NuOperator` custom resources. Use this mode for platform teams, multi-tenant clusters, and runtime script updates.

## 1. Deploy Manager

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

## 2. Install CRDs

Apply the NuOperator Custom Resource Definition:

```bash
# Apply CRDs from the repository
kubectl apply -f operator/chart/crds/nuop.yaml
```

## 3. Create NuOperator Resources

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
      path: scripts
  mappings:
    - name: configmap-controller
      group: ""
      version: v1
      kind: ConfigMap
      labelSelectors:
        app.kubernetes.io/managed-by: example-operator
```

## Sourcing CRDs from a Git Repository

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

The managed instance's ServiceAccount (named by `spec.serviceAccountName`) must carry the CRD permissions shown below.

## Startup Sequence

In Manager/Managed mode, CRDs are fetched from a source:

1. **Init container starts** with `NUOP_MODE=init`
2. **`init-sources` fetches** each `spec.sources` entry into `/sources/<path>/scripts/` or `/sources/<path>/crds/`, according to its `source_type`
3. **`install-crds` runs** in the init container if any CRDs were fetched
4. **Main container starts** the operator, which loads scripts from `/sources/`

## Managed RBAC

The managed instance's ServiceAccount needs CRD permissions when it installs CRDs from a `source_type: crds` source:

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

The Helm chart gates the CRD rule behind `rbac.crdAutoInstall` (default `false`).

## Related Documentation

- [Deployment Overview](README.md)
- [Standard Mode](standard-mode.md)
- [Automatic CRD Installation](crd-auto-installation.md)
- [Script Development Guide](../SCRIPT-DEVELOPMENT.md)
