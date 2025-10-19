# Nushell Operator

⚠️ **Work in Progress** ⚠️

This project is currently under active development and is not yet ready for production use. Breaking changes may occur frequently as we iterate on the design and implementation. Please use with caution and expect the API and functionality to evolve significantly.

## Overview

The Nushell Operator (nuop) is a Kubernetes operator that enables you to build custom controllers using [Nushell](https://www.nushell.sh/) scripts. Instead of writing controllers in traditional programming languages, you can define reconciliation logic using Nushell's powerful shell scripting capabilities.

## Key Features

- **Script-based Controllers**: Define Kubernetes controllers using Nushell scripts in a modular directory structure.
- **Dynamic Source Management**: Fetch reconciliation scripts from various sources (Git repositories, etc.)
- **Flexible Resource Mapping**: Configure which Kubernetes resources trigger which scripts using field and label selectors
- **Environment Configuration**: Supply environment variables and credentials for your scripts
- **Custom Requeue Logic**: Control when and how often reconciliation occurs

## Documentation

### [Examples](examples/README.md)
Collection of example configurations showing various nuop deployment patterns and use cases. Demonstrates environment variables, source authentication, resource mappings, and different operator configurations.

### [Example Scripts](operator/scripts/README.md)
Sample Nushell operator scripts demonstrating patterns like ConfigMap replication and Secret cloning. These serve as working examples and starting points for building custom operators.

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
Scripts are organized in directories with `mod.nu` entry points, bundled directly into container images and automatically discovered based on their metadata. Each script directory defines its target Kubernetes resource kind and reconciliation logic.

### Manager + Managed Mode
The manager operator watches for `NuOperator` custom resources and creates managed operator deployments. Each `NuOperator` CR defines:
- **Sources**: Where managed operators should fetch Nushell scripts from
- **Mappings**: Which Kubernetes resources should trigger which scripts in the managed operators
- **Environment**: Variables and configuration needed by the managed operator scripts

When a `NuOperator` CR is created or updated, the manager provisions or updates the corresponding managed operator deployment. The managed operators then watch for their mapped resources and execute the corresponding Nushell scripts for reconciliation.

## Development

### Prerequisites

- **Nix**: Install Nix with flakes support ([Determinate Systems installer](https://install.determinate.systems/) recommended)
- **direnv**: Install direnv for automatic environment activation ([installation guide](https://direnv.net/docs/installation.html))

### Quick Start

1. **Clone and enter the repository**:
   ```bash
   git clone https://github.com/ck3mp3r/nuop.git
   cd nuop
   direnv allow  # Automatically activates development environment
   ```

2. **Verify setup**:
   ```bash
   # Check tools are available
   cargo --version
   nu --version
   kind --version
   
   # Run tests
   op-tests
   ```

3. **Start local Kubernetes cluster**:
   ```bash
   kind-start  # Creates local cluster with proper configuration
   ```

### Development Environment

The project uses [devenv](https://devenv.sh/) for reproducible development environments with two configurations:

- **Default shell** (`nix develop`): Full development environment with all tools
- **CI shell** (`nix develop .#ci`): Minimal environment for testing only

#### Available Tools & Scripts

| Command | Description |
|---------|-------------|
| `op-build` | Build the operator binary |
| `op-tests` | Run all tests including integration tests |
| `op-coverage` | Generate code coverage report |
| `op-clippy` | Run Rust linter |
| `op-fmt` | Format Rust code |
| `kind-start` | Start local Kubernetes cluster |
| `op-run-standard` | Run operator in standard mode locally |
| `op-run-manager` | Run operator in manager mode locally |
| `op-run-managed` | Run operator in managed mode locally |

#### Manual Environment Setup

If you prefer not to use direnv:

```bash
# Enter development shell manually
nix develop --no-pure-eval

# Or use the minimal CI shell
nix develop .#ci --no-pure-eval
```

### Building and Testing

```bash
# Build the operator
op-build

# Run all tests (includes script execution tests)
op-tests

# Run specific test modules
cargo test --manifest-path operator/Cargo.toml config
cargo test --manifest-path operator/Cargo.toml --lib standard

# Generate coverage report
op-coverage
```

### Local Development Workflow

1. **Start local cluster**: `kind-start`
2. **Make code changes** in `operator/src/`
3. **Test changes**: `op-tests`
4. **Build container**: `op-build` 
5. **Deploy locally**: Apply example configurations to test

### Nix Flake Structure

The project provides multiple development shells:

```bash
# Full development environment (default)
nix develop

# Minimal CI environment (faster, fewer dependencies)  
nix develop .#ci

# Show all available outputs
nix flake show
```

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on code style, testing, and submitting pull requests.
