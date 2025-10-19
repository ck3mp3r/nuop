# Nushell Operator

⚠️ **Work in Progress** ⚠️

The Nushell Operator (nuop) enables you to build Kubernetes controllers using [Nushell](https://www.nushell.sh/) scripts instead of traditional programming languages.

## 📚 Documentation

- **[Complete Documentation](docs/README.md)** - Full documentation index
- **[Quick Start & Development](docs/DEVELOPMENT.md)** - Get started in minutes
- **[Script Development Guide](docs/SCRIPT-DEVELOPMENT.md)** - Write your own operators
- **[Examples](docs/examples/README.md)** - Sample configurations and patterns
- **[Development Guide](docs/DEVELOPMENT.md)** - Setup, workflow, and troubleshooting

## 🚀 Quick Start

```bash
git clone https://github.com/ck3mp3r/nuop.git
cd nuop
direnv allow    # Activates development environment
op-tests        # Run tests
```

## ✨ Key Features

- **Script-based Controllers**: Define controllers using Nushell's powerful scripting
- **Multiple Deployment Modes**: Standard (recommended), Manager, and Managed modes  
- **Flexible Resource Mapping**: Target any Kubernetes resource with selectors
- **Dynamic Sources**: Fetch scripts from Git repositories or container images

## How It Works

**Standard Mode (Recommended)**: Bundle Nushell scripts into container images. Scripts are automatically discovered and registered as controllers based on their metadata.

**Manager + Managed Mode**: Dynamic provisioning system where a manager watches `NuOperator` custom resources and creates managed operator deployments that fetch scripts from external sources.

See the [complete documentation](docs/README.md) for detailed deployment approaches and architecture.

## Contributing

See our [Contributing Guide](docs/CONTRIBUTING.md) for development setup and submission process.
