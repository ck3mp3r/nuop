FROM ghcr.io/nushell/nushell:0.116.1-alpine AS final

USER root
RUN apk add \
  git \
  github-cli \
  kubectl \
  --no-cache

COPY --from=nix-result bin/operator /bin/
COPY ./docker/entrypoint /bin/
COPY ./docker/init-sources /bin/
COPY ./docker/install-crds /bin/
RUN mkdir -p /sources && chown -R nushell:nushell /sources

USER nushell
WORKDIR /sources

ENTRYPOINT ["entrypoint"]
