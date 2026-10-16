# Contributing to Nushell Operator

Thank you for your interest in contributing to the Nushell Operator! This guide covers the contribution process: branching, code style, commit conventions, and the pull request workflow.

For development environment setup, tools, and project structure, see the [Development Guide](README.md).

## Getting Started

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
   op-tests          # Run tests
   op-build          # Package and load the operator image
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

Run the suite with `op-tests` (default profile) or `op-tests-ci` (CI profile, JUnit XML). See [testing.md](testing.md) for the test organization, nextest usage, and the mock/script-execution test patterns.

When writing tests:

- Place unit tests in separate files under `operator/test/`, mirroring the `src/` layout
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
   cluster-start
   ```

2. **Build and load image**:
   ```bash
   op-build
   kind load docker-image nuop:local --name nuop
   ```

3. **Deploy operator**:
   ```bash
   kubectl apply -f docs/examples/minimal.yaml
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

### Review Checklist

- [ ] Tests pass (`op-tests`)
- [ ] Formatting is clean (`op-fmt`)
- [ ] Linting is clean (`op-clippy`)
- [ ] New behavior is covered by tests under `operator/test/`
- [ ] Commit messages follow the conventional format
- [ ] User-facing docs are updated

## Getting Help

- **Documentation**: Check the main README and examples
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Code**: Look at existing operator scripts for patterns

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project (see [LICENSE.md](LICENSE.md)).
