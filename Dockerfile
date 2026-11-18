# syntax=docker/dockerfile:1.7

FROM worker-artifact AS compiled-worker

FROM ubuntu:24.04 AS runtime

RUN groupadd --system --gid 10001 nexora \
    && useradd --system --uid 10001 --gid 10001 --no-create-home nexora \
    && apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -m 0700 -o nexora -g nexora /var/lib/nexora

COPY --from=compiled-worker /nexora-mcp /usr/local/bin/nexora-mcp

USER 10001:10001
WORKDIR /var/lib/nexora

ENTRYPOINT ["/usr/local/bin/nexora-mcp"]
