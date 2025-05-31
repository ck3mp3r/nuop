# NuOperator Examples

This directory contains various example configurations for the NuOperator CRD.

## Examples Overview

### Basic Examples
- **basic-example.yaml** - Simple NuOperator configuration with minimal settings
- **minimal-operator.yaml** - The most minimal configuration possible

### Development Examples
- **local-development.yaml** - Configuration for local development with volume-mounted sources
- **foo-bar.yaml** - Original example with secret cloner and config replicator

### Production Examples
- **multi-resource-operator.yaml** - Complex operator managing multiple resource types with credentials
- **namespace-manager.yaml** - Operator for managing namespace provisioning
- **custom-resource-operator.yaml** - Example working with custom resources (CRDs)

### Supporting Resources
- **config-alpha.yaml** / **config-beta.yaml** - Example ConfigMaps with various labels
- **secret-alpha.yaml** / **secrets-examples.yaml** - Example Secrets for testing
- **sample-resources.yaml** - Sample Deployments, Services, Pods, and Namespaces
- **mappings/secret-cloner.yaml** - Example mapping configuration

## Key Features Demonstrated

### Environment Variables
- Static values
- ConfigMap references (`configMapKeyRef`)
- Secret references (`secretKeyRef`)
- Field references (`fieldRef`)
- Resource field references (`resourceFieldRef`)

### Source Types
- Git repositories (https://, git://) with optional authentication
- Local filesystem paths (volume mounts or container paths)
- Query parameters for git repos: `?ref=branch&dir=subdirectory`

### Source Authentication
- Username/password authentication
- Token-based authentication
- Mixed authentication methods

### Resource Mappings
- Core Kubernetes resources (Pod, Service, ConfigMap, Secret, etc.)
- Custom resources with specific groups
- Label selectors and field selectors
- Custom requeue intervals

### Advanced Configuration
- Custom images
- Service account assignment
- Multiple source repositories
- Different authentication per source

## Usage

Apply any of these examples to your cluster:

```bash
kubectl apply -f examples/basic-example.yaml
```

Make sure to adjust:
- Namespaces to match your environment
- Source repository URLs
- Credentials and secrets
- Service account names and RBAC permissions
