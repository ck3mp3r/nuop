# Test Operator: Greeting Operator
# Demonstrates automatic CRD installation feature
#
# This Dockerfile extends the base nuop image and adds:
# - Custom CRD (GreetingRequest) in /sources/greeting-operator/crds
# - Operator script in /sources/greeting-operator/scripts
#
# The CRD will be automatically installed when the container starts in Standard mode

FROM ghcr.io/ck3mp3r/nuop:latest

USER root

# Remove the default scripts from base image to avoid permission issues
RUN rm -rf /scripts/*

# Copy custom CRD - will be auto-installed on startup
COPY ./crds/ /sources/greeting-operator/crds/

# Copy only our test operator script (flattened into the operator's scripts dir)
COPY ./scripts/greeting-operator/ /sources/greeting-operator/scripts/

# Fix permissions
RUN chown -R nushell:nushell /sources

USER nushell
WORKDIR /sources

# The entrypoint from base image will:
# 1. Run install-crds (which installs CRDs from /sources/**/crds/)
# 2. Start the operator (which loads scripts from /sources)
