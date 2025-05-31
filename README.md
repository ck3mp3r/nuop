# Nushell Operator

⚠️ **Work in Progress** ⚠️

This project is currently under active development and is not yet ready for production use. Breaking changes may occur frequently as we iterate on the design and implementation. Please use with caution and expect the API and functionality to evolve significantly.

## Overview

The Nushell Operator (nuop) is a Kubernetes operator that enables you to build custom controllers using [Nushell](https://www.nushell.sh/) scripts. Instead of writing controllers in traditional programming languages, you can define reconciliation logic using Nushell's powerful shell scripting capabilities.

## Key Features

- **Script-based Controllers**: Define Kubernetes controllers using Nushell scripts, or any executable scripts for that matter.
- **Dynamic Source Management**: Fetch reconciliation scripts from various sources (Git repositories, etc.)
- **Flexible Resource Mapping**: Configure which Kubernetes resources trigger which scripts using field and label selectors
- **Environment Configuration**: Supply environment variables and credentials for your scripts
- **Custom Requeue Logic**: Control when and how often reconciliation occurs

## Deployment Approach

The Nushell Operator is designed with **Standard Mode** as the primary deployment approach, offering the most straightforward and efficient way to run script-based controllers.

### Recommended: Standard Mode with Custom Images (`NUOP_MODE=standard` or default)

**Build custom container images from the base nuop image** - this is the recommended approach that provides:

- **Complete Independence**: Runs without any manager oversight or external dependencies
- **Self-Contained Controllers**: Bundle your Nushell scripts directly into the container image
- **Automatic Discovery**: Scripts are automatically discovered and registered as controllers based on their metadata
- **Simplified Deployment**: Deploy directly to Kubernetes with standard Deployment manifests
- **Resource Definition**: Each script defines its own target Kubernetes resource kind through metadata
- **No Runtime Dependencies**: No need for external script sources or dynamic provisioning

This approach eliminates complexity while providing maximum control and reliability. Simply extend the base nuop image with your scripts and deploy.

### Alternative: Manager + Managed Mode Coordination

The Manager and Managed modes work together as a two-tier system for dynamic controller provisioning:

#### Manager Mode (`NUOP_MODE=manager`)
- Acts as a meta-operator that watches `NuOperator` custom resources
- Creates and manages deployments for each `NuOperator` instance
- Each managed deployment runs in `NUOP_MODE=managed`
- Handles dynamic provisioning and lifecycle management of script-based controllers
- Useful for multi-tenant environments where different teams need dynamic controller provisioning

#### Managed Mode (`NUOP_MODE=managed`)
- Runs as a worker instance spawned and controlled by the manager
- Requires both mapping configurations and script files to be present
- Maps scripts to Kubernetes resources based on explicit mapping definitions from the `NuOperator` CR
- Fetches scripts from configured sources (Git repositories, etc.)
- Executes the actual reconciliation logic for the mapped Kubernetes resources

**How they work together**: The manager watches for `NuOperator` CRs and creates deployments running in managed mode. Each managed instance handles the reconciliation for the specific resource mappings defined in its corresponding `NuOperator` CR.

## How it Works

### Standard Mode
Scripts are bundled directly into container images and automatically discovered based on their metadata. Each script defines its target Kubernetes resource kind and reconciliation logic.

### Manager + Managed Mode
The manager operator watches for `NuOperator` custom resources and creates managed operator deployments. Each `NuOperator` CR defines:
- **Sources**: Where managed operators should fetch Nushell scripts from
- **Mappings**: Which Kubernetes resources should trigger which scripts in the managed operators
- **Environment**: Variables and configuration needed by the managed operator scripts

When a `NuOperator` CR is created or updated, the manager provisions or updates the corresponding managed operator deployment. The managed operators then watch for their mapped resources and execute the corresponding Nushell scripts for reconciliation.
