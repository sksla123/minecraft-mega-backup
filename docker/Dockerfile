# syntax=docker/dockerfile:1.7

# ==========================================
# Stage 1: Build mcmgb
# ==========================================
FROM rust:1-slim-bookworm AS rust-builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release \
    && cp target/release/mcmgb /mcmgb


# ==========================================
# Stage 2: Runtime
# ==========================================
FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="mcmgb"
LABEL org.opencontainers.image.description="Minecraft backup daemon with automatic MEGA synchronization"
LABEL org.opencontainers.image.source="https://github.com/sksla123/mcmgb"
LABEL org.opencontainers.image.licenses="MIT"

RUN --mount=type=cache,id=apt-final-cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-final-lib,target=/var/lib/apt,sharing=locked \
    rm -f /var/lib/apt/lists/lock /var/cache/apt/archives/lock \
    && apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        libc6 \
        libstdc++6 \
        zlib1g \
    && curl -fsSL \
        -o /tmp/megacmd.deb \
        https://mega.nz/linux/repo/Debian_12/amd64/megacmd-Debian_12_amd64.deb \
    && apt-get install -y --no-install-recommends /tmp/megacmd.deb \
    && rm -f /tmp/megacmd.deb \
    && apt-get purge -y curl \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=rust-builder /mcmgb /usr/bin/mcmgb
COPY docker/entrypoint.sh /usr/bin/entrypoint.sh

RUN chmod +x \
    /usr/bin/mcmgb \
    /usr/bin/entrypoint.sh

WORKDIR /app

ENTRYPOINT ["/usr/bin/entrypoint.sh"]

