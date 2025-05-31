# nuop
Nushell Operator

## Overview

The Nushell Operator (nuop) is a Kubernetes operator that enables you to build custom controllers using [Nushell](https://www.nushell.sh/) scripts. Instead of writing controllers in traditional programming languages, you can define reconciliation logic using Nushell's powerful shell scripting capabilities.

## Key Features

- **Script-based Controllers**: Define Kubernetes controllers using Nushell scripts, or any executable scripts for that matter.
- **Dynamic Source Management**: Fetch reconciliation scripts from various sources (Git repositories, etc.)
- **Flexible Resource Mapping**: Configure which Kubernetes resources trigger which scripts using field and label selectors
- **Environment Configuration**: Supply environment variables and credentials for your scripts
- **Custom Requeue Logic**: Control when and how often reconciliation occurs

## Operator Modes

The Nushell Operator supports multiple deployment modes controlled by the `NUOP_MODE` environment variable:

### Manager Mode (`NUOP_MODE=manager`)
- Acts as a meta-operator that watches `NuOperator` custom resources
- Creates and manages deployments for each `NuOperator` instance
- Handles dynamic provisioning of script-based controllers
- Ideal for multi-tenant environments where different teams manage their own controllers

### Managed Mode (`NUOP_MODE=managed`)
- Runs as a worker instance spawned by the manager
- Requires both mapping configurations and script files to be present
- Maps scripts to Kubernetes resources based on explicit mapping definitions
- Used when you want centralized control over which scripts handle which resources

### Standard Mode (`NUOP_MODE=standard` or default)
- Runs independently without manager oversight
- Automatically discovers and registers controllers based on script metadata
- Each script defines its own target Kubernetes resource kind
- Simplest deployment model for standalone use cases

## How it Works

The operator watches for `NuOperator` custom resources that define:
- **Sources**: Where to fetch your Nushell scripts from
- **Mappings**: Which Kubernetes resources should trigger which scripts
- **Environment**: Variables and configuration needed by your scripts

When a mapped resource changes, the operator executes the corresponding Nushell script to handle the reconciliation.
