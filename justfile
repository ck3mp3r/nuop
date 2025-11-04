# Nushell Operator Justfile

# Variables
IMAGE_NAME := "nuop"
VERSION := "latest"
REGISTRY := "ghcr.io/ck3mp3r"

# List all available recipes
default:
    @just --list

# Run all tests
tests:
    cd operator && cargo test

# Build docker image and load into kind
build:
    cd operator && docker build --debug -f docker/Dockerfile . -t {{REGISTRY}}/{{IMAGE_NAME}}:{{VERSION}}
    kind load docker-image {{REGISTRY}}/{{IMAGE_NAME}}:latest -n nuop

# Build multi-platform images with buildx
buildx:
    cd operator && docker buildx create --name mybuilder --use || true
    cd operator && docker buildx inspect --bootstrap
    cd operator && docker buildx build -f docker/Dockerfile --platform linux/amd64,linux/arm64 -t {{REGISTRY}}/{{IMAGE_NAME}}:{{VERSION}} --push .

# Clean build artifacts
clean:
    cd operator && cargo clean

# Run clippy linter
clippy:
    cd operator && cargo clippy

# Format code with rustfmt
fmt:
    cd operator && cargo fmt

# Generate CRD YAML
crds:
    cd operator && cargo run --bin generate > chart/crds/nuop.yaml

# Generate code coverage report
coverage:
    cd operator && cargo tarpaulin --out Html

# Run GitHub Actions test workflow locally with act
act-test:
    @act push \
        --rm \
        --container-options "--network bridge --dns 8.8.8.8 --dns 1.1.1.1" \
        --container-architecture linux/aarch64 \
        -s GITHUB_TOKEN="${GITHUB_TOKEN:-}" \
        -s ACTIONS_RUNTIME_TOKEN="${GITHUB_TOKEN:-}" \
        -P ubuntu-latest=catthehacker/ubuntu:js-latest \
        -W .github/workflows/test.yaml \
        -j test

# Run GitHub Actions buildx workflow locally with act
act-buildx:
    @act workflow-dispatch \
        --rm \
        --container-architecture linux/aarch64 \
        --privileged \
        --container-daemon-socket /var/run/docker.sock \
        -s GITHUB_TOKEN="${GITHUB_TOKEN:-}" \
        -s ACTIONS_RUNTIME_TOKEN="${GITHUB_TOKEN:-}" \
        -P ubuntu-latest=catthehacker/ubuntu:js-latest \
        -W .github/workflows/buildx.yaml \
        -j build
