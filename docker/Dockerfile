# ==========================================
# docker/Dockerfile
# ==========================================
# Stage 1: Rust (mcmgb) 빌더
FROM rust:1-slim-bookworm AS rust-builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp target/release/mcmgb /mcmgb


# Stage 2: 최종 실행 이미지 (MEGA 공식 .deb 패키지 직접 설치)
FROM debian:bookworm-slim

# 필요 패키지 및 MEGA 공식 deb 패키지 설치
RUN --mount=type=cache,id=apt-final-cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-final-lib,target=/var/lib/apt,sharing=locked \
    rm -f /var/lib/apt/lists/lock /var/cache/apt/archives/lock && \
    apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    ca-certificates curl libc6 libstdc++6 zlib1g \
    && curl -fsSL -o /tmp/megacmd.deb https://mega.nz/linux/repo/Debian_12/amd64/megacmd-Debian_12_amd64.deb \
    && apt-get install -y --no-install-recommends /tmp/megacmd.deb \
    && rm -f /tmp/megacmd.deb \
    && rm -rf /var/lib/apt/lists/*

# Rust 빌드 산출물 및 Entrypoint 복사
COPY --from=rust-builder /mcmgb /usr/bin/mcmgb
RUN chmod +x /usr/bin/mcmgb

COPY docker/entrypoint.sh /usr/bin/entrypoint.sh
RUN chmod +x /usr/bin/entrypoint.sh

WORKDIR /app
ENTRYPOINT ["/usr/bin/entrypoint.sh"]
