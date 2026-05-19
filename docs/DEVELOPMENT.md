# Development Guide

This guide covers setting up your development environment and working with the Nushell Operator codebase.

## Prerequisites

- **Nix**: Install Nix with flakes support ([Determinate Systems installer](https://install.determinate.systems/) recommended)
- **direnv**: Install direnv for automatic environment activation ([installation guide](https://direnv.net/docs/installation.html))

## Quick Start

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

## Development Environment

The project uses [devenv](https://devenv.sh/) for reproducible development environments with two configurations:

- **Default shell** (`nix develop`): Full development environment with all tools
- **CI shell** (`nix develop .#ci`): Minimal environment for testing only

### Available Tools & Scripts

| Command | Description |
|---------|-------------|
| `op-build` | Build the operator binary |
| `op-tests` | Run all tests including integration tests |
| `op-coverage` | Generate code coverage report |
| `op-clippy` | Run Rust linter |
| `op-fmt` | Format Rust code |

| `op-run-standard` | Run operator in standard mode locally |
| `op-run-manager` | Run operator in manager mode locally |
| `op-run-managed` | Run operator in managed mode locally |

### Manual Environment Setup

If you prefer not to use direnv:

```bash
# Enter development shell manually
nix develop --no-pure-eval

# Or use the minimal CI shell
nix develop .#ci --no-pure-eval
```

## Building and Testing

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

## Local Development Workflow

1. **Start local cluster**: `kind-start`
2. **Make code changes** in `operator/src/`
3. **Test changes**: `op-tests`
4. **Build container**: `op-build`
5. **Deploy locally**: Apply example configurations to test

## Nix Flake Structure

The project provides multiple development shells:

```bash
# Full development environment (default)
nix develop

# Minimal CI environment (faster, fewer dependencies)
nix develop .#ci

# Show all available outputs
nix flake show
```

## Project Structure

```
operator/
├── src/nuop/           # Main operator code
│   ├── manager/        # Manager mode implementation
│   ├── reconciler/     # Core reconciliation logic
│   ├── config.rs       # Configuration handling
│   └── ...
├── scripts/            # Example operator scripts
├── docker/             # Container build files
└── nix/               # Nix build configuration
```

## Troubleshooting

### Script Issues

**Test scripts locally:**
```bash
# Test script configuration
echo '{}' | nu operator/scripts/your-script/mod.nu config

# Test with sample resource
cat test-resource.yaml | nu operator/scripts/your-script/mod.nu reconcile
```

**Common script errors:**
- **"Script not found"** - Ensure `mod.nu` exists in script directory
- **"Invalid configuration"** - Check script config returns all required fields (name, group, version, kind)
- **Exit code issues** - Scripts should exit 0 (no changes) or 2 (changes made)

### General Kubernetes Troubleshooting

For general Kubernetes debugging, see:
- [Kubernetes Troubleshooting Guide](https://kubernetes.io/docs/tasks/debug/)
- [Debugging Pods](https://kubernetes.io/docs/tasks/debug/debug-application/debug-pods/)
- [Debugging Services](https://kubernetes.io/docs/tasks/debug/debug-application/debug-service/)

## Contributing

See [Contributing Guidelines](CONTRIBUTING.md) for detailed information on code style, testing, and submitting pull requests.