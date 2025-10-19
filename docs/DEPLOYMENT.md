# Production Deployment Guide

This guide covers best practices for deploying the Nushell Operator in production environments.

## Deployment Modes

### Standard Mode (Recommended)

Deploy self-contained operators with scripts bundled into container images.

**Advantages**:
- No external dependencies
- Simplified deployment
- Better security and isolation
- Faster startup times
- Easier version management

**Use Case**: Most production deployments where you have a specific set of operators to deploy.

### Manager + Managed Mode

Deploy a manager that dynamically provisions operators based on `NuOperator` custom resources.

**Advantages**:
- Dynamic operator provisioning
- Multi-tenant environments
- Centralized management
- Runtime script updates

**Use Case**: Platform teams managing multiple operators or multi-tenant environments.

## Standard Mode Deployment

### 1. Create Custom Operator Image

```dockerfile
# Dockerfile
FROM ghcr.io/ck3mp3r/nuop:latest

# Copy your operator scripts
COPY scripts/ /scripts/

# Optional: Add any additional dependencies
# RUN apk add --no-cache curl jq
```

Build and push your image:
```bash
docker build -t your-registry/your-operator:v1.0.0 .
docker push your-registry/your-operator:v1.0.0
```

### 2. Create RBAC Resources

```yaml
# rbac.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: your-operator
  namespace: your-namespace
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: your-operator
rules:
# Add permissions for resources your scripts manage
- apiGroups: [""]
  resources: ["configmaps", "secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["namespaces"]
  verbs: ["get", "list", "watch"]
# Add any additional permissions needed
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: your-operator
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: your-operator
subjects:
- kind: ServiceAccount
  name: your-operator
  namespace: your-namespace
```

### 3. Deploy Operator

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: your-operator
  namespace: your-namespace
  labels:
    app.kubernetes.io/name: your-operator
    app.kubernetes.io/version: v1.0.0
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: your-operator
  template:
    metadata:
      labels:
        app.kubernetes.io/name: your-operator
    spec:
      serviceAccountName: your-operator
      containers:
      - name: operator
        image: your-registry/your-operator:v1.0.0
        env:
        - name: RUST_LOG
          value: info
        - name: NUOP_MODE
          value: standard
        - name: NUOP_SCRIPT_PATH
          value: /scripts
        resources:
          requests:
            memory: 64Mi
            cpu: 100m
          limits:
            memory: 256Mi
            cpu: 500m
        securityContext:
          allowPrivilegeEscalation: false
          runAsNonRoot: true
          runAsUser: 1000
          capabilities:
            drop:
            - ALL
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

## Manager + Managed Mode Deployment

### 1. Deploy Manager

```yaml
# manager-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nuop-manager
  namespace: nuop-system
spec:
  replicas: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: nuop-manager
  template:
    metadata:
      labels:
        app.kubernetes.io/name: nuop-manager
    spec:
      serviceAccountName: nuop-manager
      containers:
      - name: manager
        image: ghcr.io/ck3mp3r/nuop:latest
        env:
        - name: NUOP_MODE
          value: manager
        - name: RUST_LOG
          value: info
        resources:
          requests:
            memory: 128Mi
            cpu: 100m
          limits:
            memory: 512Mi
            cpu: 1000m
```

### 2. Install CRDs

```bash
kubectl apply -f https://github.com/ck3mp3r/nuop/releases/latest/download/crds.yaml
```

### 3. Create NuOperator Resources

```yaml
# operators.yaml
apiVersion: kemper.buzz/v1alpha1
kind: NuOperator
metadata:
  name: config-replicator
  namespace: nuop-system
spec:
  serviceAccountName: config-replicator
  sources:
  - location: https://github.com/your-org/operators.git?ref=v1.0.0
    path: /scripts
  mappings:
  - name: config-replicator
    kind: ConfigMap
    version: v1
    labelSelectors:
      replicate: "true"
  env:
  - name: LOG_LEVEL
    value: debug
```

## Security Best Practices

### 1. RBAC Configuration

**Principle of Least Privilege**: Grant only required permissions.

```yaml
# Example: ConfigMap replicator permissions
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: [""]
  resources: ["namespaces"]
  verbs: ["get", "list", "watch"]
# Do NOT grant "*" permissions unless absolutely necessary
```

### 2. Container Security

```yaml
securityContext:
  allowPrivilegeEscalation: false
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  capabilities:
    drop:
    - ALL
```

### 3. Network Policies

Restrict network access where possible:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: your-operator-netpol
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: your-operator
  policyTypes:
  - Ingress
  - Egress
  egress:
  # Allow Kubernetes API access
  - to: []
    ports:
    - protocol: TCP
      port: 443
  # Allow DNS
  - to: []
    ports:
    - protocol: UDP
      port: 53
```

### 4. Image Security

- Use specific image tags, not `latest`
- Scan images for vulnerabilities
- Use minimal base images
- Sign container images when possible

```yaml
image: ghcr.io/ck3mp3r/nuop:v0.2.0  # Specific version
# NOT: ghcr.io/ck3mp3r/nuop:latest
```

## Resource Management

### 1. Resource Limits

Set appropriate resource requests and limits:

```yaml
resources:
  requests:
    memory: 64Mi   # Minimum required
    cpu: 100m      # 0.1 CPU cores
  limits:
    memory: 256Mi  # Maximum allowed
    cpu: 500m      # 0.5 CPU cores
```

### 2. Horizontal Pod Autoscaling

For high-traffic operators:

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: your-operator-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: your-operator
  minReplicas: 1
  maxReplicas: 5
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### 3. Pod Disruption Budgets

Ensure availability during updates:

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: your-operator-pdb
spec:
  minAvailable: 1
  selector:
    matchLabels:
      app.kubernetes.io/name: your-operator
```

## Monitoring and Observability

### 1. Logging Configuration

Configure structured logging:

```yaml
env:
- name: RUST_LOG
  value: info,your_operator=debug
- name: LOG_FORMAT
  value: json  # For log aggregation
```

### 2. Metrics Collection

If using Prometheus:

```yaml
# ServiceMonitor for Prometheus Operator
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: your-operator
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: your-operator
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

### 3. Health Checks

Implement proper health checks:

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 30
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10
  timeoutSeconds: 3
  failureThreshold: 3
```

## High Availability

### 1. Multi-Region Deployment

For critical operators, deploy across regions:

```yaml
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchLabels:
            app.kubernetes.io/name: your-operator
        topologyKey: topology.kubernetes.io/zone
```

### 2. Leader Election

For operators that need coordination:

```yaml
env:
- name: ENABLE_LEADER_ELECTION
  value: "true"
- name: LEADER_ELECTION_NAMESPACE
  valueFrom:
    fieldRef:
      fieldPath: metadata.namespace
```

## Backup and Disaster Recovery

### 1. Configuration Backup

Backup your operator configurations:

```bash
# Backup operator deployments
kubectl get deployments -l app.kubernetes.io/managed-by=nuop -o yaml > operator-backup.yaml

# Backup NuOperator CRs (for manager mode)
kubectl get nuoperators -o yaml > nuoperators-backup.yaml
```

### 2. State Recovery

Most nuop operators are stateless, but ensure:
- RBAC permissions are restored
- Dependent resources (ConfigMaps, Secrets) are available
- Network connectivity is established

## Update Strategies

### 1. Rolling Updates

Configure safe rolling updates:

```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxUnavailable: 0
    maxSurge: 1
```

### 2. Blue-Green Deployment

For critical operators:

```bash
# Deploy new version with different label
kubectl apply -f operator-v2.yaml

# Verify new version works
kubectl get pods -l version=v2

# Switch traffic by updating selector
kubectl patch service your-operator -p '{"spec":{"selector":{"version":"v2"}}}'

# Remove old version
kubectl delete deployment your-operator-v1
```

## Performance Tuning

### 1. Requeue Intervals

Optimize requeue timing based on your needs:

```yaml
mappings:
- name: frequent-reconcile
  requeue_after_noop: 30    # 30 seconds for frequent checks
- name: infrequent-reconcile
  requeue_after_noop: 3600  # 1 hour for stable resources
```

### 2. Resource Selectors

Use specific selectors to reduce event volume:

```yaml
mappings:
- name: config-replicator
  labelSelectors:
    app.kubernetes.io/replicate: "true"
    environment: production      # More specific
  # Instead of watching all ConfigMaps
```

### 3. Script Optimization

- Minimize external API calls in scripts
- Use efficient data structures
- Cache frequently accessed data
- Avoid expensive operations in reconcile loops

## Troubleshooting Production Issues

### 1. Common Issues

**High CPU/Memory Usage**:
- Check requeue intervals
- Review script efficiency
- Monitor event volume

**Failed Reconciliations**:
- Check RBAC permissions
- Verify script execution
- Review resource quotas

**Slow Performance**:
- Optimize resource selectors
- Reduce reconciliation frequency
- Scale horizontally if needed

### 2. Debugging Tools

```bash
# Check operator logs
kubectl logs -l app.kubernetes.io/name=your-operator -f

# Monitor resource usage
kubectl top pods -l app.kubernetes.io/name=your-operator

# Check events
kubectl get events --sort-by='.lastTimestamp'

# Describe problematic resources
kubectl describe deployment your-operator
```

## Migration Strategies

### From Development to Production

1. **Test thoroughly** in staging environment
2. **Use specific image tags** instead of latest
3. **Configure appropriate resources** based on testing
4. **Set up monitoring** before deployment
5. **Plan rollback strategy** before deployment

### Version Upgrades

1. **Review release notes** for breaking changes
2. **Test in staging** environment first
3. **Backup configurations** before upgrade
4. **Use rolling updates** for minimal downtime
5. **Monitor closely** after upgrade

## Best Practices Summary

✅ **Security**:
- Use minimal RBAC permissions
- Run as non-root user
- Use specific image tags
- Implement network policies

✅ **Reliability**:
- Set resource limits
- Configure health checks
- Use pod disruption budgets
- Plan for high availability

✅ **Observability**:
- Enable structured logging
- Monitor resource usage
- Set up alerting
- Regular health checks

✅ **Performance**:
- Optimize selectors
- Tune requeue intervals
- Monitor and scale appropriately
- Efficient script design

✅ **Operations**:
- Automate deployments
- Plan update strategies
- Backup configurations
- Document procedures