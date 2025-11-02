FROM ghcr.io/nushell/nushell:0.108.0-alpine AS final

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
COPY ./scripts /scripts

RUN chown -R nushell:nushell /scripts

USER nushell
WORKDIR /scripts

ENTRYPOINT ["entrypoint"]
