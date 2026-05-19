# Test Operator: Greeting Operator
# Demonstrates automatic CRD installation feature
#
# This Dockerfile extends the base nuop image and adds:
# - Custom CRD (GreetingRequest) in /crds directory
# - Operator script in /scripts directory
#
# The CRD will be automatically installed when the container starts in Standard mode

FROM ghcr.io/ck3mp3r/nuop:latest

USER root

# Remove the default scripts from base image to avoid permission issues
RUN rm -rf /scripts/*

# Copy custom CRD - will be auto-installed on startup
COPY ./crds/ /crds/

# Copy only our test operator script
COPY ./scripts/ /scripts/

# Fix permissions
RUN chown -R nushell:nushell /scripts /crds

USER nushell
WORKDIR /scripts

# The entrypoint from base image will:
# 1. Run install-crds (which installs CRDs from /crds)
# 2. Start the operator (which loads scripts from /scripts)
