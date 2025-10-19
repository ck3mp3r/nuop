# Contributing to Nushell Operator

Thank you for your interest in contributing to the Nushell Operator! This guide will help you get started with development and understand our contribution process.

## Development Setup

### Prerequisites

- **Nix**: Install Nix with flakes support ([Determinate Systems installer](https://install.determinate.systems/) recommended)
- **direnv**: Install direnv for automatic environment activation ([installation guide](https://direnv.net/docs/installation.html))

### Getting Started

1. **Fork and clone the repository**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/nuop.git
   cd nuop
   ```

2. **Set up development environment**:
   ```bash
   direnv allow  # Automatically activates development environment
   ```

3. **Verify setup**:
   ```bash
   cargo --version
   nu --version
   op-tests  # Run all tests
   ```

## Development Workflow

### Making Changes

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/bug-description
   ```

2. **Make your changes** and test them:
   ```bash
   op-tests          # Run all tests
   op-clippy         # Check for linting issues
   op-fmt            # Format code
   ```

3. **Test locally with Kind**:
   ```bash
   kind-start        # Start local Kubernetes cluster
   op-build          # Build operator
   # Apply test configurations and verify functionality
   ```

### Code Style

- **Rust code**: Follow standard Rust formatting (`cargo fmt`)
- **Nushell scripts**: Use consistent indentation and naming
- **Commit messages**: Use conventional commits format
  - `feat: add new feature`
  - `fix: resolve bug in reconciler`
  - `docs: update README`
  - `refactor: improve error handling`
  - `test: add integration tests`

### Testing

#### Running Tests

```bash
# Run all tests (including integration tests)
op-tests

# Run specific test modules
cargo test --manifest-path operator/Cargo.toml config
cargo test --manifest-path operator/Cargo.toml reconciler
cargo test --manifest-path operator/Cargo.toml manager

# Run tests with specific patterns
cargo test --manifest-path operator/Cargo.toml test_reconcile

# Generate coverage report
op-coverage
```

#### Test Categories

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test script execution and Kubernetes interactions
- **Controller tests**: Test full reconciliation loops with mock Kubernetes API

#### Writing Tests

- Place unit tests in the same file as the code being tested
- Use descriptive test names that explain the scenario
- Include both positive and negative test cases
- Mock external dependencies when possible

### Building and Packaging

```bash
# Build the operator binary
op-build

# Build for different architectures (uses Nix cross-compilation)
nix build .#operator-x86_64-linux
nix build .#operator-aarch64-linux

# Build container image
docker build -f operator/docker/Dockerfile operator/ -t nuop:local
```

### Local Testing with Kubernetes

1. **Start local cluster**:
   ```bash
   kind-start
   ```

2. **Build and load image**:
   ```bash
   op-build
   kind load docker-image nuop:local --name nuop
   ```

3. **Deploy operator**:
   ```bash
   kubectl apply -f docs/examples/basic-example.yaml
   ```

4. **Check logs**:
   ```bash
   kubectl logs -l app.kubernetes.io/name=nuop -f
   ```

## Pull Request Process

### Before Submitting

- [ ] All tests pass (`op-tests`)
- [ ] Code is properly formatted (`op-fmt`)
- [ ] No linting issues (`op-clippy`)
- [ ] Documentation is updated if needed
- [ ] Commit messages follow conventional format

### Submitting a PR

1. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create pull request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Reference any related issues
   - Include testing steps if applicable

3. **Respond to feedback** and make necessary changes

### PR Guidelines

- **Single responsibility**: Each PR should address one feature or fix
- **Small and focused**: Prefer smaller PRs for easier review
- **Good commit history**: Use meaningful commit messages
- **Documentation**: Update docs for user-facing changes
- **Tests**: Include tests for new functionality

## Code Organization

### Project Structure

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

### Key Components

- **Reconciler**: Core logic for watching and reconciling resources
- **Manager**: Manages multiple operator instances (manager mode)
- **Config**: Handles script discovery and configuration
- **Scripts**: Example Nushell operator implementations

## Development Environment Details

### Available Commands

| Command | Description |
|---------|-------------|
| `op-build` | Build operator binary |
| `op-tests` | Run all tests |
| `op-coverage` | Generate code coverage |
| `op-clippy` | Run Rust linter |
| `op-fmt` | Format Rust code |
| `op-crds` | Generate CRD manifests |
| `kind-start` | Start local K8s cluster |
| `op-run-*` | Run operator in different modes |

### Development Shells

- **Default** (`nix develop`): Full development environment
- **CI** (`nix develop .#ci`): Minimal environment for testing

### Troubleshooting Development Issues

- **Nix/direnv issues**: Try `direnv reload` or restart your shell
- **Test failures**: Check that `kind-start` has been run for integration tests
- **Build issues**: Ensure you're in the development shell (`direnv status`)

## Getting Help

- **Documentation**: Check the main README and examples
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Code**: Look at existing operator scripts for patterns

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project (see [LICENSE.md](LICENSE.md)).