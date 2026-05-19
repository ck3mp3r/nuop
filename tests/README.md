# Greeting Operator - Test for Automatic CRD Installation

This directory contains a complete test operator that demonstrates the **automatic CRD installation** feature introduced in this PR.

## Overview

The **Greeting Operator** is a simple custom operator that:

1. Watches for `GreetingRequest` custom resources
2. Creates ConfigMaps with greeting messages in different languages
3. Demonstrates automatic CRD installation on startup

### What It Tests

- ✅ **Automatic CRD Installation**: CRDs are auto-installed from `/crds` directory
- ✅ **Custom Resource Reconciliation**: Operator watches and reconciles custom resources
- ✅ **Standard Mode Operation**: Operator runs in standard mode with script-based controller
- ✅ **Status Updates**: Updates custom resource status with greeting information
- ✅ **Finalizers**: Cleans up ConfigMaps when GreetingRequest is deleted

## Directory Structure

```
tests/
├── Dockerfile                              # Extends base nuop image
├── crds/
│   └── greetingrequest-crd.yaml           # Custom CRD (auto-installed)
├── scripts/
│   └── greeting-operator/
│       └── mod.nu                         # Operator script
├── examples/
│   ├── greetingrequest.yaml               # Example custom resources
│   └── deployment.yaml                    # Full deployment with RBAC
└── README.md                              # This file
```

## Custom Resource Definition

**Group**: `demo.nuop.io`  
**Kind**: `GreetingRequest`  
**Version**: `v1`

### Spec Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Name of the person/thing to greet |
| `language` | string | No | Language for greeting (`en`, `es`, `fr`, `de`, `ja`) |
| `style` | string | No | Greeting style (`formal`, `informal`) |

### Status Fields

| Field | Type | Description |
|-------|------|-------------|
| `greeting` | string | The generated greeting message |
| `configMapName` | string | Name of the created ConfigMap |
| `lastUpdated` | string | Timestamp of last update |
| `state` | string | Current state (`Pending`, `Ready`, `Failed`) |
| `message` | string | Human-readable status message |

## How It Works

### 1. Container Startup Sequence

```
Container starts → install-crds runs → CRDs installed → operator starts
```

The `install-crds` script automatically:
- Scans `/crds` directory for YAML files
- Installs or updates each CRD
- Waits for CRDs to be established
- Then allows operator to start

### 2. Reconciliation Logic

When a `GreetingRequest` is created/updated:

1. **Read spec**: Extract name, language, and style
2. **Generate greeting**: Create greeting message in specified language
3. **Create/Update ConfigMap**: Store greeting in a ConfigMap
4. **Update status**: Set status with greeting and ConfigMap name

### 3. Finalization

When a `GreetingRequest` is deleted:

1. **Delete ConfigMap**: Remove the associated greeting ConfigMap
2. **Remove finalizer**: Operator framework removes finalizer automatically

## Quick Start

### Prerequisites

- Kind cluster running (use `kind-start` from devenv)
- Base nuop image built (use `just build`)
- kubectl configured for kind cluster

### Build and Deploy

```bash
# Build the base operator image first
just build

# Build the test operator image
just test-build

# Deploy the operator
just test-deploy

# Or run everything at once
just test-run
```

### Watch the Operator

```bash
# Watch operator logs
kubectl logs -f -l app=greeting-operator -n default

# Check if CRD was auto-installed
kubectl get crd greetingrequests.demo.nuop.io

# Check operator deployment
kubectl get deployment greeting-operator -n default
```

### Create Test Resources

```bash
# Create example greeting requests
kubectl apply -f tests/examples/greetingrequest.yaml

# Watch the custom resources
kubectl get greetingrequests -w

# Check the generated ConfigMaps
kubectl get configmaps -l app.kubernetes.io/managed-by=greeting-operator

# View a greeting
kubectl get configmap greeting-hello-world -o yaml
```

### Example Usage

Create a greeting request:

```yaml
apiVersion: demo.nuop.io/v1
kind: GreetingRequest
metadata:
  name: hello-alice
  namespace: default
spec:
  name: Alice
  language: en
  style: informal
```

The operator will:
1. Auto-install the CRD (if not present)
2. Create a ConfigMap named `greeting-hello-alice`
3. Store the greeting: "Hey Alice! Nice to meet you!"
4. Update the status with greeting information

Check the results:

```bash
# View the GreetingRequest status
kubectl get greetingrequest hello-alice -o yaml

# View the created ConfigMap
kubectl get configmap greeting-hello-alice -o yaml
```

## Testing Different Languages

The operator supports multiple languages:

```bash
# English (formal)
kubectl apply -f - <<EOF
apiVersion: demo.nuop.io/v1
kind: GreetingRequest
metadata:
  name: formal-english
spec:
  name: Sir Knight
  language: en
  style: formal
EOF

# Spanish (informal)
kubectl apply -f - <<EOF
apiVersion: demo.nuop.io/v1
kind: GreetingRequest
metadata:
  name: spanish-friend
spec:
  name: Amigo
  language: es
  style: informal
EOF

# Japanese (formal)
kubectl apply -f - <<EOF
apiVersion: demo.nuop.io/v1
kind: GreetingRequest
metadata:
  name: japanese-sensei
spec:
  name: Sensei
  language: ja
  style: formal
EOF
```

## Cleanup

```bash
# Delete all test resources
just test-clean

# Or manually:
kubectl delete -f tests/examples/deployment.yaml
kubectl delete greetingrequests --all
kubectl delete configmaps -l app.kubernetes.io/managed-by=greeting-operator
```

## Troubleshooting

### CRD Not Installing

Check operator logs for CRD installation output:

```bash
kubectl logs -l app=greeting-operator | head -n 50
```

You should see:
```
🔍 Checking for CRDs to install...
📦 Found 1 CRD file(s) to process
📄 Processing: /crds/greetingrequest-crd.yaml
✅ CRD greetingrequests.demo.nuop.io installed successfully
```

### Permission Errors

Ensure the ServiceAccount has CRD permissions:

```bash
kubectl auth can-i create customresourcedefinitions \
  --as=system:serviceaccount:default:greeting-operator
```

### Operator Not Reconciling

Check if the CRD is established:

```bash
kubectl get crd greetingrequests.demo.nuop.io -o jsonpath='{.status.conditions[?(@.type=="Established")].status}'
```

Should return `True`.

## Extending This Test

You can use this test as a template for your own operators:

1. **Modify the CRD**: Edit `crds/greetingrequest-crd.yaml` with your spec
2. **Update the script**: Edit `scripts/greeting-operator/mod.nu` with your logic
3. **Rebuild**: Run `just test-build`
4. **Test**: Run `just test-deploy`

## Architecture

```
┌─────────────────────────────────────┐
│   Greeting Operator Container       │
│                                     │
│  ┌──────────────────────────────┐  │
│  │  /crds/                      │  │
│  │    greetingrequest-crd.yaml │  │  Auto-installed
│  └──────────────────────────────┘  │  on startup
│                                     │
│  ┌──────────────────────────────┐  │
│  │  /scripts/greeting-operator/ │  │
│  │    mod.nu                    │  │  Loaded by
│  └──────────────────────────────┘  │  operator
│                                     │
│  ┌──────────────────────────────┐  │
│  │  Operator (Rust binary)      │  │
│  │  - Watches GreetingRequests  │  │
│  │  - Runs Nushell script       │  │
│  │  - Updates status            │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
           │
           ├─────────► Kubernetes API
           │           - GreetingRequests
           │           - ConfigMaps
           └─────────► Updates Status
```

## Related Documentation

- [CRD Auto-Installation Guide](../docs/CRD-AUTO-INSTALLATION.md)
- [Script Development Guide](../docs/SCRIPT-DEVELOPMENT.md)
- [Deployment Guide](../docs/DEPLOYMENT.md)
