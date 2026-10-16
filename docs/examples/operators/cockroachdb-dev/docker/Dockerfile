# CockroachDB Dev Operator — Standard Mode Image
#
# Extends the base nuop image with the cockroachdb-dev operator script and the
# CrdbCluster CRD. The CRD is sourced from the cockroach-operator repository and
# installed automatically on startup by install-crds.
#
# Build from the repository root, so the script is in the build context:
#   docker build -t cockroachdb-dev -f docs/examples/operators/cockroachdb-dev/docker/Dockerfile docs/examples/operators/cockroachdb-dev/

FROM ghcr.io/ck3mp3r/nuop:latest

USER root

# Operator script — copied from the build context
COPY mod.nu /sources/cockroachdb-dev/scripts/mod.nu

# CrdbCluster CRD — fetched from the cockroach-operator repository
RUN mkdir -p /sources/cockroachdb-dev/crds \
 && wget -q -O /sources/cockroachdb-dev/crds/crds.yaml \
      https://github.com/cockroachdb/cockroach-operator/raw/master/install/crds.yaml

RUN chown -R nushell:nushell /sources

USER nushell
WORKDIR /sources
