# Documentation Index

Documentation for the Nushell Operator (nuop). Start with the section that matches your role.

## For Users

Deploy and operate nuop, and write operator scripts.

### Concepts

- **[Architecture](architecture.md)** — system design, components, and how the operator reconciles resources

### Deployment

- **[Deployment Overview](deployment/README.md)** — choose between Standard mode and Manager + Managed mode
- **[Standard Mode](deployment/standard-mode.md)** — scripts bundled into a container image (recommended)
- **[Manager + Managed Mode](deployment/managed-mode.md)** — dynamic provisioning from `NuOperator` custom resources
- **[Automatic CRD Installation](deployment/crd-auto-installation.md)** — install CRDs from `/sources/**/crds/` before controllers start

### Scripting

- **[Script Development Tutorial](scripting/README.md)** — write your first operator script
- **[Script API Reference](scripting/api-reference.md)** — config/reconcile/finalize protocol, exit codes, environment variables
- **[Script Examples](scripting/examples.md)** — working scripts you can copy

### Examples

- **[Example Configurations](examples/README.md)** — complete manifests for Standard and Managed modes

## For Developers

Contribute to the nuop codebase.

- **[Development Guide](development/README.md)** — environment setup, tools, and workflow
- **[Testing](development/testing.md)** — test organization and how to run the suite
- **[Code Coverage](development/coverage.md)** — coverage tooling and CI integration
- **[Contributing](development/contributing.md)** — PR process, code style, and commit conventions

## Reference

- **[NuOperator CRD](api/nuoperator-crd.md)** — custom resource definition reference
- **[Script API Reference](scripting/api-reference.md)** — script protocol and configuration fields

## External Resources

- **[Nushell Documentation](https://www.nushell.sh/book/)** — learn Nushell scripting
- **[Kubernetes Operator Guide](https://kubernetes.io/docs/concepts/extend-kubernetes/operator/)** — understanding operators
- **[Custom Resource Definitions](https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/)** — CRD concepts
