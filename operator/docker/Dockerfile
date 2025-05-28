FROM docker.io/nixos/nix:2.29.0 AS builder

WORKDIR /tmp/build

COPY ./ /tmp/build

RUN nix \
  --extra-experimental-features "nix-command flakes" \
  --option filter-syscalls false \
  build

FROM ghcr.io/nushell/nushell:0.104.1-alpine AS final

USER root
RUN apk add \
  git \
  github-cli \
  kubectl \
  --no-cache


COPY --from=builder /tmp/build/result/bin/operator /bin/
COPY ./docker/entrypoint /bin/
COPY ./docker/init-sources /bin/
COPY ./scripts /scripts

USER root
RUN chown -R nushell:nushell /scripts

USER nushell
WORKDIR /scripts

ENTRYPOINT ["entrypoint"]
