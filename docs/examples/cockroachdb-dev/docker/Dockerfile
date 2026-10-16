# CockroachDB Dev Operator — Standard Mode Image
#
# Extends the base nuop image with the cockroachdb-dev operator script and the
# CrdbCluster CRD. The CRD is sourced from the CockroachDB operator Helm charts
# repository and installed automatically on startup by install-crds.
#
# Build from the repository root, so the script is in the build context:
#   docker build -t cockroachdb-dev -f docs/examples/cockroachdb-dev/docker/Dockerfile docs/examples/cockroachdb-dev/

FROM ghcr.io/ck3mp3r/nuop:latest

USER root

# Operator script — copied from the build context
COPY scripts/mod.nu /sources/cockroachdb-dev/scripts/mod.nu

# CrdbCluster CRD — fetched from the CockroachDB operator Helm charts repository
RUN mkdir -p /sources/cockroachdb-dev/crds \
 && wget -q -O /sources/cockroachdb-dev/crds/crds.yaml \
      https://raw.githubusercontent.com/cockroachdb/helm-charts/master/cockroachdb-operator/charts/operator/manifests/crds/crdb.cockroachlabs.com_crdbclusters.yaml

RUN chown -R nushell:nushell /sources

USER nushell
WORKDIR /sources
