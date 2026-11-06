# Pinned build environment for TilEm for reMarkable (aarch64-unknown-linux-musl, static).
# Same Rust/zig toolchain as the chessMarkable RPP 1.0.1 release:
# rustc 1.94.1, cargo-zigbuild 0.22.1, zig 0.15.2. libclang is added for bindgen.
FROM rust:1.94.1-slim-bookworm@sha256:cf9dd0ec73e75f827fe59123fff9dc65af1a1c8363c3c31ee8d7f8ad0b6a5fb2
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl xz-utils \
 && rm -rf /var/lib/apt/lists/*
ARG ZIG=0.15.2
RUN curl -fsSL https://ziglang.org/download/${ZIG}/zig-aarch64-linux-${ZIG}.tar.xz | tar -xJ -C /opt \
 && ln -s /opt/zig-aarch64-linux-${ZIG}/zig /usr/local/bin/zig
RUN cargo install cargo-zigbuild --version 0.22.1 --locked \
 && rustup target add aarch64-unknown-linux-musl
RUN apt-get update \
 && apt-get install -y --no-install-recommends libclang-dev \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /build
