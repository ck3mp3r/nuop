# CockroachDB Dev Operator

The CockroachDB Dev Operator reconciles `CrdbCluster` resources (`crdb.cockroachlabs.com/v1alpha1`) for local development. It is a ready-to-use Nushell operator script for the Nushell Operator (nuop).

It is a lightweight alternative to the deprecated [CockroachDB Kubernetes operator](https://github.com/cockroachdb/cockroach-operator). It watches the same CRD and produces the same external interface, so Helm charts that target a CockroachDB cluster work the same in development and production.

## Requirements

- The `CrdbCluster` CRD (`crdbclusters.crdb.cockroachlabs.com`) installed in the cluster. The CRD is sourced from the cockroach-operator repository, not from this repository.
- RBAC that allows the operator to read and patch `crdbclusters` and `crdbclusters/status`, and to manage `statefulsets` and `services`.
- The Nushell runtime and `kubectl` on the operator's PATH.

The script is mode-agnostic. It does not know whether it runs in Standard or Managed mode.

## Created Resources

| Resource | Name | Details |
|----------|------|---------|
| StatefulSet | `<metadata.name>` | One pod per `spec.nodes` replica, container `db` |
| Discovery Service | `<metadata.name>` | Headless (`clusterIP: None`), `publishNotReadyAddresses: true` |
| Public Service | `<metadata.name>-public` | `ClusterIP` |

Both services expose the same three ports:

| Port Name | Default | CLI Flag |
|-----------|---------|----------|
| `grpc` | 26258 | `--listen-addr` |
| `http` | 8080 | `--http-port` |
| `sql` | 26257 | `--sql-addr` |

The StatefulSet runs this command:

```
/cockroach/cockroach.sh start \
  --advertise-host=$(POD_NAME).<metadata.name>.<namespace> \
  --insecure \
  --http-port=<httpPort> \
  --sql-addr=:<sqlPort> \
  --listen-addr=:<grpcPort> \
  --logtostderr=INFO \
  --cache=256MiB \
  --max-sql-memory=256MiB \
  [<spec.additionalArgs>] \
  --join=<name>-0.<name>.<namespace>:<grpcPort>,<name>-1.<name>.<namespace>:<grpcPort>,<name>-2.<name>.<namespace>:<grpcPort>
```

The join string always seeds the first three pods. The script omits the default join string when `spec.additionalArgs` contains a `--join` argument.

## Spec Fields

The script reads these fields and ignores all others:

| Field | Default | Effect |
|-------|---------|--------|
| `spec.image.name` | `cockroachdb/cockroach:v23.1.11` | Container image |
| `spec.nodes` | 3 | StatefulSet replicas |
| `spec.grpcPort` | 26258 | gRPC port on both services and the container |
| `spec.httpPort` | 8080 | HTTP port on both services and the container |
| `spec.sqlPort` | 26257 | SQL port on both services and the container |
| `spec.dataStore.pvc.spec` | emptyDir | `volumeClaimTemplates` for the `datadir` volume |
| `spec.additionalArgs` | `[]` | Appended to the `cockroach start` command |
| `spec.resources` | unset | Container resource requests and limits |
| `spec.additionalLabels` | `{}` | Merged into all resource labels and the pod selector |

When `spec.dataStore.pvc.spec` is absent, the `datadir` volume is an `emptyDir`.

## Status Fields

The script patches the `status` subresource:

| Field | Values |
|-------|--------|
| `status.clusterStatus` | `Starting` while the StatefulSet is not fully ready, `Finished` when all replicas are ready |
| `status.crdbcontainerimage` | The value of `spec.image.name` |

## Labels

The script applies the same labels as the full operator, so resources are indistinguishable from the full operator's output:

| Label Key | Value |
|-----------|-------|
| `app.kubernetes.io/name` | `cockroachdb` |
| `app.kubernetes.io/instance` | `<metadata.name>` |
| `app.kubernetes.io/component` | `database` |
| `app.kubernetes.io/part-of` | `cockroachdb` |
| `app.kubernetes.io/managed-by` | `cockroach-operator` |

`spec.additionalLabels` is merged on top. The pod selector uses `name`, `instance`, `component`, and any `spec.additionalLabels`.

## Finalizers

The script uses a finalizer to ensure cleanup when a `CrdbCluster` is deleted:

- `github.com/ck3mp3r-nuop-crdb-dev-finalizer`

Finalization deletes the StatefulSet and both Services. It does not delete PVCs, so data survives a cluster deletion.

## Requeue Behavior

The operator requeues after 60 seconds when no changes are detected, so drift is corrected periodically.

## Script Implementation

The `mod.nu` script follows the common operator pattern:

1. **Configuration**: The `main config` function returns operator metadata including the script name, API group, version, target kind, finalizer identifier, and requeue timing.

2. **Reconciliation**: The `main reconcile` function handles create/update events. It builds the three desired manifests, patches `status.clusterStatus` to `Starting`, applies the manifests, patches `status.crdbcontainerimage`, and patches `status.clusterStatus` to `Finished` when all replicas are ready. It returns exit code 0 (no change) or 2 (changed).

3. **Finalization**: The `main finalize` function handles deletion events. It deletes the StatefulSet and both Services and returns exit code 0.

### Execution Model

Scripts are executed via the Nushell interpreter:

```bash
nu docs/examples/operators/cockroachdb-dev/mod.nu config      # Get configuration
nu --stdin docs/examples/operators/cockroachdb-dev/mod.nu reconcile   # Handle resource changes
nu --stdin docs/examples/operators/cockroachdb-dev/mod.nu finalize    # Handle resource deletion
```

In containers, scripts are at `/sources/<op-name>/scripts/mod.nu`.

## Example CrdbCluster

```yaml
apiVersion: crdb.cockroachlabs.com/v1alpha1
kind: CrdbCluster
metadata:
  name: crdb-no-topo
  namespace: default
spec:
  nodes: 3
  image:
    name: cockroachdb/cockroach:v23.1.11
  dataStore:
    pvc:
      spec:
        accessModes: [ReadWriteOnce]
        resources:
          requests:
            storage: 60Gi
        volumeMode: Filesystem
```

Apply the resource and check the result:

```bash
kubectl apply -f crdb-no-topo.yaml
kubectl get statefulset crdb-no-topo
kubectl get service crdb-no-topo crdb-no-topo-public
kubectl get crdbcluster crdb-no-topo -o jsonpath='{.status}'
```

Connect to the cluster:

```bash
kubectl exec -it crdb-no-topo-0 -- /cockroach/cockroach sql --insecure
```

## Limitations

This script targets local development. It differs from the full operator in these ways:

- Insecure mode only. TLS, certificates, and secure mode are not supported.
- Fixed `--cache` and `--max-sql-memory` of 256MiB.
- No security context, no `preStop` drain hook, no init containers.
- No topology, ingress, node selectors, tolerations, or affinity.
- No rolling upgrade orchestration or version checks.
- No connection Secrets or ConfigMaps.

## Usage with Nushell Operator

This script is designed to work with the Nushell Operator in Standard Mode. In the container image it is copied to `/sources/<op-name>/scripts/` and discovered by the operator scanning `/sources/`.

To use this script:

1. Include it in your custom operator image
2. Install the `CrdbCluster` CRD and the required RBAC
3. Deploy the image to your Kubernetes cluster
4. Apply a `CrdbCluster` resource

The operator will automatically detect and manage the resource.
