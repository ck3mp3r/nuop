# Troubleshooting Guide

This guide covers common issues and debugging techniques for the Nushell Operator.

## Development Environment Issues

### Nix/direnv Problems

**Problem**: `direnv: error loading`
```bash
# Solution: Reload direnv configuration
direnv reload

# If that doesn't work, allow the directory again
direnv allow

# Check direnv status
direnv status
```

**Problem**: Commands not found (`cargo`, `nu`, etc.)
```bash
# Check if you're in the development shell
echo $IN_NIX_SHELL  # Should output "1"

# Manual shell entry if direnv isn't working
nix develop --no-pure-eval

# Verify tools are available
which cargo nu kind kubectl
```

**Problem**: `nix develop` fails with evaluation errors
```bash
# Check flake syntax
nix flake check

# Update flake inputs
nix flake update

# Try with fresh evaluation
nix develop --no-pure-eval --refresh
```

### Test Failures

**Problem**: Integration tests fail with "connection refused"
```bash
# Start local Kubernetes cluster first
kind-start

# Verify cluster is running
kubectl cluster-info

# Check if kubeconfig is set correctly
echo $KUBECONFIG  # Should show kind/kube.config path
```

**Problem**: Script execution tests fail
```bash
# Check if nushell is available
nu --version

# Verify script directory structure
ls -la operator/scripts/*/mod.nu

# Test script execution manually
echo '{}' | nu operator/scripts/config-replicator/mod.nu config
```

## Operator Deployment Issues

### Pod Startup Problems

**Problem**: Operator pod won't start
```bash
# Check pod status and events
kubectl get pods -l app.kubernetes.io/name=nuop
kubectl describe pod <pod-name>

# Check operator logs
kubectl logs -l app.kubernetes.io/name=nuop --tail=50

# Common issues:
# - Image pull failures
# - RBAC permission issues
# - Invalid configuration
```

**Problem**: CrashLoopBackOff
```bash
# Get recent logs before crashes
kubectl logs -l app.kubernetes.io/name=nuop --previous

# Check for common causes:
# - Missing RBAC permissions
# - Invalid script configuration
# - Script execution errors
# - Resource constraints
```

### RBAC and Permissions

**Problem**: "Forbidden" errors in logs
```bash
# Check if ServiceAccount exists
kubectl get serviceaccount nuop-operator

# Verify ClusterRole permissions
kubectl describe clusterrole nuop-operator

# Check ClusterRoleBinding
kubectl describe clusterrolebinding nuop-operator

# Test permissions manually
kubectl auth can-i get configmaps --as=system:serviceaccount:default:nuop-operator
```

### Script Discovery Issues

**Problem**: Scripts not found or loaded
```bash
# Check script directory in container
kubectl exec -it <operator-pod> -- ls -la /scripts

# Verify script configuration
kubectl exec -it <operator-pod> -- nu /operator/scripts/<script>/mod.nu config

# Check operator mode
kubectl logs -l app.kubernetes.io/name=nuop | grep "Running in.*mode"
```

## Script Execution Issues

### Script Errors

**Problem**: Script execution fails
```bash
# Test script locally first
echo '{"test": "data"}' | nu operator/scripts/your-script/mod.nu reconcile

# Check script syntax
nu --check operator/scripts/your-script/mod.nu

# Debug with verbose output
RUST_LOG=debug kubectl logs -l app.kubernetes.io/name=nuop -f
```

**Problem**: Invalid JSON output
```bash
# Test script output format
echo '{}' | nu operator/scripts/your-script/mod.nu reconcile | jq .

# Common issues:
# - Extra print statements in reconcile function
# - Invalid JSON structure
# - Missing JSON output
# - Wrong exit codes
```

**Problem**: Script timeouts
```bash
# Check for infinite loops in script
# Add timeout to external commands
# Optimize script performance
# Increase operator timeout if necessary
```

### Resource Processing Issues

**Problem**: Resources not being processed
```bash
# Check label selectors match
kubectl get <resource-type> --show-labels

# Verify operator is watching correct namespace
kubectl logs -l app.kubernetes.io/name=nuop | grep "Watching"

# Test label selector
kubectl get configmaps -l "app.kubernetes.io/replicate=yes"
```

**Problem**: Duplicate processing or loops
```bash
# Check for missing management labels
kubectl get <resource> -o yaml | grep "app.kubernetes.io/managed-by"

# Verify finalizer handling
kubectl get <resource> -o yaml | grep finalizers

# Look for requeue loops in logs
kubectl logs -l app.kubernetes.io/name=nuop | grep -i requeue
```

## Performance Issues

### High CPU/Memory Usage

**Problem**: Operator consuming too many resources
```bash
# Check resource usage
kubectl top pods -l app.kubernetes.io/name=nuop

# Monitor specific metrics
kubectl logs -l app.kubernetes.io/name=nuop | grep -E "(processing|reconcile)"

# Common causes:
# - Too frequent requeuing
# - Large resources being processed
# - Inefficient scripts
# - Memory leaks in scripts
```

### Slow Reconciliation

**Problem**: Slow response to resource changes
```bash
# Check requeue intervals in script config
echo '{}' | nu operator/scripts/your-script/mod.nu config | jq .requeueAfterSeconds

# Monitor reconciliation timing
kubectl logs -l app.kubernetes.io/name=nuop | grep -E "reconcile.*took"

# Optimize scripts:
# - Reduce external calls
# - Use more specific selectors
# - Cache frequently accessed data
```

## Common Error Messages

### "Script not found"
```
Error: Script directory '/operator/scripts/my-script' does not contain mod.nu
```
**Solution**: Ensure script directory has `mod.nu` file with correct structure

### "Invalid configuration"
```
Error: Script configuration missing required field 'kind'
```
**Solution**: Check script's `main config` function returns all required fields

### "JSON parse error"
```
Error: Failed to parse script output as JSON
```
**Solution**: Ensure script outputs valid JSON and doesn't have extra print statements

### "Exit code 1"
```
Warning: Script exited with code 1
```
**Solution**: Check script for errors, use exit codes 0 or 2 only

### "Finalizer timeout"
```
Error: Finalizer did not complete within timeout
```
**Solution**: Check finalizer logic removes finalizer from resource

## Debugging Techniques

### Local Script Testing

```bash
# Test script configuration
echo '{}' | nu operator/scripts/your-script/mod.nu config

# Test reconcile with sample resource
cat test-resource.json | nu operator/scripts/your-script/mod.nu reconcile

# Test finalize
cat deleting-resource.json | nu operator/scripts/your-script/mod.nu finalize
```

### Log Analysis

```bash
# Follow operator logs
kubectl logs -l app.kubernetes.io/name=nuop -f

# Filter for specific script
kubectl logs -l app.kubernetes.io/name=nuop | grep "your-script"

# Check for errors only
kubectl logs -l app.kubernetes.io/name=nuop | grep -i error

# Export logs for analysis
kubectl logs -l app.kubernetes.io/name=nuop --since=1h > operator.log
```

### Resource Inspection

```bash
# Check resource status and events
kubectl describe <resource-type> <resource-name>

# Look for operator annotations
kubectl get <resource-type> <resource-name> -o yaml | grep -A5 annotations

# Check finalizers
kubectl get <resource-type> <resource-name> -o yaml | grep -A5 finalizers
```

### Performance Profiling

```bash
# Monitor resource usage over time
kubectl top pods -l app.kubernetes.io/name=nuop --containers

# Check reconciliation frequency
kubectl logs -l app.kubernetes.io/name=nuop | grep "Reconciling" | wc -l

# Analyze requeue patterns
kubectl logs -l app.kubernetes.io/name=nuop | grep -o "Requeue.*" | sort | uniq -c
```

## Getting Help

### Information to Gather

When reporting issues, include:

1. **Environment**:
   - Kubernetes version (`kubectl version`)
   - Operator version/image tag
   - Operating mode (standard/manager/managed)

2. **Configuration**:
   - Operator deployment YAML
   - Script configuration output
   - Resource manifests causing issues

3. **Logs**:
   - Operator logs (`kubectl logs`)
   - Kubernetes events (`kubectl get events`)
   - Script test output

4. **Resources**:
   - Resource definitions being processed
   - Current resource status
   - Expected vs actual behavior

### Useful Commands for Debugging

```bash
# Complete environment snapshot
kubectl get all -l app.kubernetes.io/name=nuop
kubectl describe deployment nuop-operator
kubectl logs -l app.kubernetes.io/name=nuop --tail=100
kubectl get events --sort-by='.lastTimestamp' | tail -20

# Script-specific debugging
for script in operator/scripts/*/; do
  echo "Testing $script"
  echo '{}' | nu "$script/mod.nu" config
done

# Resource monitoring
kubectl get configmaps,secrets -l "app.kubernetes.io/replicate=yes" -w
```

### Community Resources

- **GitHub Issues**: Search existing issues or create new ones
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check examples and script documentation
- **Source Code**: Review operator source and example scripts