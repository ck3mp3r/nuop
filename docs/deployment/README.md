# Deployment

nuop supports two deployment modes. Pick the one that matches how you manage operator scripts.

| Mode | Scripts come from | Best for |
|------|-------------------|----------|
| [Standard mode](standard-mode.md) | Baked into a custom container image | Most production deployments with a fixed set of operators |
| [Manager + Managed mode](managed-mode.md) | Fetched at runtime from git sources declared on a `NuOperator` CR | Platform teams, multi-tenant clusters, runtime script updates |

Both modes share [automatic CRD installation](crd-auto-installation.md): CRD manifests under `/sources/**/crds/` are installed and waited on before controllers start.

## Standard Mode (Recommended)

Deploy self-contained operators with scripts bundled into container images.

**Advantages**:
- No external dependencies
- Simplified deployment
- Better security and isolation
- Faster startup times
- Easier version management

**Use Case**: Most production deployments where you have a specific set of operators to deploy.

See [standard-mode.md](standard-mode.md).

## Manager + Managed Mode

Deploy a manager that dynamically provisions operators based on `NuOperator` custom resources.

**Advantages**:
- Dynamic operator provisioning
- Multi-tenant environments
- Centralized management
- Runtime script updates

**Use Case**: Platform teams managing multiple operators or multi-tenant environments.

See [managed-mode.md](managed-mode.md).

## Shared Features

- [Automatic CRD Installation](crd-auto-installation.md) — install CRDs from `/sources/**/crds/` before controllers start
- [Script Development Guide](../scripting/README.md) — writing operator scripts
- [Examples](../examples/README.md) — complete working examples
