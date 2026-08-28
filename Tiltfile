# Tiltfile for nuop local development

# Ensure the kind cluster is running
local_resource(
    'kind-cluster',
    cmd='kind-start',
    deps=['kind/kind-cluster.yaml'],
)

# Build the operator image (compiles Rust inside a linux container via colima)
docker_build(
    'ghcr.io/ck3mp3r/nuop',
    'operator',
    dockerfile='operator/docker/Dockerfile.dev',
    only=[
        'src',
        'Cargo.toml',
        'Cargo.lock',
        'docker',
        'scripts',
    ],
)

# Create the namespace
k8s_yaml('kind/namespace.yaml')

# Deploy the manager operator via the helm chart
# (watches NuOperator CRs and generates managed child deployments)
k8s_yaml(helm(
    'operator/chart',
    name='nuop',
    namespace='nuop',
    set=['deployment.nuopMode=manager'],
))

# Deploy a managed NuOperator example (config-replicator)
# k8s_custom_deploy injects the locally-built image into spec.image
k8s_custom_deploy(
    'nuop-managed',
    apply_cmd='sed "s|ghcr.io/ck3mp3r/nuop:latest|$TILT_IMAGE_0|" tilt/managed-operator.yaml | kubectl apply -f - -o yaml',
    delete_cmd='kubectl delete -f tilt/managed-operator.yaml --ignore-not-found',
    deps=['tilt/managed-operator.yaml'],
    image_deps=['ghcr.io/ck3mp3r/nuop'],
)

# Deploy example resources to demonstrate replication
k8s_yaml('tilt/example-resources.yaml')
