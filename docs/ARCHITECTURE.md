# Architecture Overview

This document provides a comprehensive overview of the Nushell Operator (nuop) architecture, covering its design principles, component interactions, and operational modes.

## System Overview

The Nushell Operator enables building Kubernetes controllers using Nushell scripts instead of traditional programming languages. It bridges the gap between Kubernetes APIs and shell scripting by providing a framework for script-based reconciliation.

```
┌─────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                      │
│                                                             │
│  ┌─────────────────┐    ┌─────────────────┐                │
│  │   Resources     │    │   nuop          │                │
│  │  (ConfigMaps,   │◄──►│   Operator      │                │
│  │   Secrets, etc) │    │   Pod(s)        │                │
│  └─────────────────┘    └─────────────────┘                │
│                                 │                           │
│                                 ▼                           │
│                    ┌─────────────────┐                      │
│                    │   Nushell       │                      │
│                    │   Scripts       │                      │
│                    │  (mod.nu files) │                      │
│                    └─────────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Operator Runtime

The core Rust application built on kube-rs that provides:

- **Kubernetes API Integration**: Watches and manages Kubernetes resources
- **Script Execution Engine**: Invokes Nushell scripts for reconciliation logic
- **Resource Management**: Handles finalizers, status updates, and error handling
- **Configuration Discovery**: Automatically discovers and loads operator scripts

**Key Files**:
- `src/main.rs` - Application entry point and mode selection
- `src/nuop/reconciler/` - Core reconciliation logic
- `src/nuop/config.rs` - Script discovery and configuration

### 2. Script Framework

Nushell scripts organized in a modular directory structure:

```
scripts/
├── operator-name/
│   ├── mod.nu          # Required entry point
│   ├── helpers.nu      # Optional helper functions
│   └── config.nu       # Optional configuration utilities
```

Each script implements three main functions:
- `main config` - Returns operator metadata and configuration
- `main reconcile` - Handles resource create/update events  
- `main finalize` - Handles resource deletion events

### 3. Configuration System

**Static Configuration** (Standard Mode):
- Scripts bundled into container images
- Configuration defined in script metadata
- No external dependencies

**Dynamic Configuration** (Manager + Managed Mode):
- `NuOperator` CRDs define script sources and mappings
- Runtime script fetching from Git repositories
- Dynamic resource mapping configuration

## Operational Modes

### Standard Mode (Recommended)

```
┌─────────────────────────────────────────────────────────────┐
│                 Standard Mode Operator                     │
│                                                             │
│  ┌─────────────────┐    ┌─────────────────┐                │
│  │ Script Discovery│    │ Resource        │                │
│  │ /scripts/*/     │───►│ Controllers     │                │
│  │ mod.nu          │    │ (one per script)│                │
│  └─────────────────┘    └─────────────────┘                │
│                                 │                           │
│                                 ▼                           │
│              ┌─────────────────────────────────────┐        │
│              │        Kubernetes API               │        │
│              │    (ConfigMaps, Secrets, CRDs)      │        │
│              └─────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

**Flow**:
1. Operator starts and scans `/scripts` directory
2. For each script directory containing `mod.nu`:
   - Executes `nu mod.nu config` to get metadata
   - Creates a controller for the specified resource kind
   - Starts watching resources matching the configuration
3. When resources change:
   - Executes `nu mod.nu reconcile` with resource JSON on stdin
   - Processes script output and exit codes
   - Updates resource based on script results

**Advantages**:
- Self-contained deployment
- No external dependencies
- Predictable behavior
- Faster startup

### Manager + Managed Mode

```
┌─────────────────────────────────────────────────────────────┐
│                    Manager Mode                            │
│                                                             │
│  ┌─────────────────┐    ┌─────────────────┐                │
│  │   NuOperator    │    │     Manager     │                │
│  │   CRD Watch     │───►│   Controller    │                │
│  │                 │    │                 │                │
│  └─────────────────┘    └─────────────────┘                │
│                                 │                           │
│                                 ▼                           │
│              ┌─────────────────────────────────────┐        │
│              │        Managed Deployments          │        │
│              │    (one per NuOperator CR)          │        │
│              └─────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Managed Mode (Per Instance)              │
│                                                             │
│  ┌─────────────────┐    ┌─────────────────┐                │
│  │ Script Sources  │    │ Resource        │                │
│  │ (Git repos,     │───►│ Controllers     │                │
│  │  volumes)       │    │ (per mapping)   │                │
│  └─────────────────┘    └─────────────────┘                │
│                                 │                           │
│                                 ▼                           │
│              ┌─────────────────────────────────────┐        │
│              │        Kubernetes API               │        │
│              │    (mapped resources only)          │        │
│              └─────────────────────────────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

**Flow**:
1. Manager watches for `NuOperator` CRDs
2. For each `NuOperator`:
   - Creates a managed deployment with specific configuration
   - Injects source and mapping configuration
3. Managed instances:
   - Fetch scripts from configured sources
   - Create controllers based on mappings
   - Execute reconciliation for mapped resources

**Advantages**:
- Dynamic script deployment
- Multi-tenant isolation
- Centralized management
- Runtime updates

## Script Execution Model

### 1. Configuration Phase

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Operator      │    │    Nushell      │    │   Script        │
│   Startup       │───►│   Interpreter   │───►│   mod.nu        │
│                 │    │                 │    │   config        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Configuration  │
                       │  JSON Response  │
                       │  (name, kind,   │
                       │   selectors)    │
                       └─────────────────┘
```

### 2. Reconciliation Phase

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Resource      │    │    Nushell      │    │   Script        │
│   Change Event  │───►│   Interpreter   │───►│   mod.nu        │
│   (JSON)        │    │   (--stdin)     │    │   reconcile     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Updated        │
                       │  Resource JSON  │
                       │  + Exit Code    │
                       │  (0=noop,2=chg) │
                       └─────────────────┘
```

### 3. Finalization Phase

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Resource      │    │    Nushell      │    │   Script        │
│   Deletion      │───►│   Interpreter   │───►│   mod.nu        │
│   Event (JSON)  │    │   (--stdin)     │    │   finalize      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Cleaned        │
                       │  Resource JSON  │
                       │  (finalizer     │
                       │   removed)      │
                       └─────────────────┘
```

## Resource Flow

### Standard Mode Resource Flow

```
1. Resource Created/Updated
   │
   ▼
2. Controller Detects Change
   │
   ▼
3. Execute Script: nu mod.nu reconcile < resource.json
   │
   ▼
4. Process Script Output
   ├─ Exit Code 0: No changes needed
   ├─ Exit Code 2: Apply changes from stdout
   └─ Exit Code 1: Log error and requeue
   │
   ▼
5. Update Resource Status
   │
   ▼
6. Schedule Next Reconciliation (if configured)
```

### Manager + Managed Mode Resource Flow

```
1. NuOperator CRD Created/Updated
   │
   ▼
2. Manager Controller Creates/Updates Deployment
   │
   ▼
3. Managed Instance Starts
   │
   ▼
4. Fetch Scripts from Sources
   │
   ▼
5. Create Controllers per Mapping
   │
   ▼
6. Standard Reconciliation Flow
```

## Data Structures

### Script Configuration

```nushell
{
  name: "script-name",           # Unique identifier
  kind: "ConfigMap",             # K8s resource kind
  apiVersion: "v1",              # API version
  group: "",                     # API group (optional)
  labelSelector: {               # Resource filtering
    "app": "my-app"
  },
  finalizer: "example.com/finalizer",  # Cleanup handling
  requeueAfterSeconds: 300       # Requeue interval
}
```

### Resource Processing

**Input** (stdin):
```json
{
  "apiVersion": "v1",
  "kind": "ConfigMap",
  "metadata": {
    "name": "example",
    "namespace": "default",
    "labels": {...},
    "annotations": {...}
  },
  "data": {...}
}
```

**Output** (stdout):
```json
{
  "apiVersion": "v1",
  "kind": "ConfigMap",
  "metadata": {
    "name": "example",
    "namespace": "default",
    "labels": {...},
    "annotations": {...}
  },
  "data": {...}
}
```

## Error Handling

### Script Execution Errors

1. **Exit Code 1**: Script error
   - Log error message
   - Requeue with backoff
   - Update resource status if possible

2. **Invalid JSON Output**: Parse error
   - Log parse error
   - Requeue with backoff
   - No resource changes applied

3. **Script Timeout**: Execution timeout
   - Kill script process
   - Log timeout error
   - Requeue with backoff

### Controller Errors

1. **Kubernetes API Errors**: 
   - Network failures: Retry with exponential backoff
   - Permission errors: Log and skip resource
   - Resource conflicts: Retry with fresh resource version

2. **Script Discovery Errors**:
   - Missing mod.nu: Log warning and skip directory
   - Invalid configuration: Log error and skip script
   - Permission errors: Log error and fail startup

## Performance Characteristics

### Resource Usage

**Standard Mode**:
- Memory: ~64MB base + script execution overhead
- CPU: Low baseline, spikes during reconciliation
- Storage: Container image size only

**Manager + Managed Mode**:
- Memory: Manager ~128MB + (Managed instances × ~64MB)
- CPU: Manager overhead + per-instance reconciliation
- Storage: Script fetching and caching

### Scalability

**Resource Watching**:
- Each script creates one controller
- Controllers use Kubernetes watch APIs (efficient)
- Resource filtering reduces event volume

**Reconciliation**:
- Scripts executed serially per resource
- Concurrent processing across different resources
- No built-in rate limiting (relies on Kubernetes)

### Bottlenecks

1. **Script Execution**: Nushell interpreter startup overhead
2. **Resource Updates**: Kubernetes API rate limits
3. **Git Fetching**: Network latency for script sources (managed mode)

## Security Model

### Process Isolation

- Scripts executed in subprocess (process isolation)
- No shared state between script executions
- Resource access limited by Kubernetes RBAC

### Container Security

- Runs as non-root user (UID 1000)
- Read-only root filesystem support
- Minimal container attack surface

### RBAC Integration

- ServiceAccount-based permissions
- Principle of least privilege
- Resource-specific access controls

### Script Security

- No direct filesystem access (beyond container)
- Environment variables for configuration
- Kubernetes client credentials via ServiceAccount

## Extension Points

### Custom Scripts

- Implement required functions: config, reconcile, finalize
- Use helper modules for complex logic
- Follow exit code conventions

### Resource Types

- Core Kubernetes resources (Pods, Services, etc.)
- Custom Resource Definitions (CRDs)
- Cluster-scoped and namespaced resources

### Integration Patterns

- External API integration via HTTP calls
- Database operations via connection tools
- File processing via volume mounts
- Secret management via Kubernetes Secrets

## Future Architecture Considerations

### Planned Enhancements

1. **Health Endpoints**: HTTP server for liveness/readiness probes
2. **Metrics Exposure**: Prometheus metrics for observability
3. **Leader Election**: High availability support
4. **Webhooks**: Admission and conversion webhook support
5. **Script Validation**: Static analysis and testing framework

### Scalability Improvements

1. **Parallel Execution**: Concurrent script execution
2. **Resource Caching**: Intelligent caching strategies
3. **Event Filtering**: Advanced filtering to reduce noise
4. **Batch Processing**: Group similar operations

### Security Enhancements

1. **Script Sandboxing**: Enhanced process isolation
2. **Code Signing**: Verify script authenticity
3. **Audit Logging**: Comprehensive operation logging
4. **Network Policies**: Restrict script network access

This architecture enables powerful script-based Kubernetes controllers while maintaining the flexibility and simplicity that makes Nushell attractive for automation tasks.