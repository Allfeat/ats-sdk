# Build from the repo root:
#   docker build -f ats-proof-tools/Dockerfile -t ats-proof-tools .

# ── Build stage ──
FROM rust:bookworm AS builder

# wasm target + trunk + wasm-opt (arch-aware)
RUN rustup target add wasm32-unknown-unknown \
    && apt-get update && apt-get install -y --no-install-recommends binaryen \
    && rm -rf /var/lib/apt/lists/* \
    && ARCH=$(uname -m) \
    && case "$ARCH" in \
         x86_64)  TRUNK_ARCH="x86_64-unknown-linux-gnu" ;; \
         aarch64) TRUNK_ARCH="aarch64-unknown-linux-gnu" ;; \
         *)       echo "unsupported arch: $ARCH" && exit 1 ;; \
       esac \
    && wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-${TRUNK_ARCH}.tar.gz" \
       | tar xzf - -C /usr/local/bin

WORKDIR /build

# Copy the ats-sdk crate (path dependency)
COPY ats-sdk/Cargo.toml ats-sdk/Cargo.toml
COPY ats-sdk/src ats-sdk/src

# Copy ats-proof-tools
COPY ats-proof-tools/Cargo.toml ats-proof-tools/Cargo.toml
COPY ats-proof-tools/Cargo.lock ats-proof-tools/Cargo.lock
COPY ats-proof-tools/Trunk.toml ats-proof-tools/Trunk.toml
COPY ats-proof-tools/index.html ats-proof-tools/index.html
COPY ats-proof-tools/src ats-proof-tools/src
COPY ats-proof-tools/style ats-proof-tools/style

WORKDIR /build/ats-proof-tools
RUN trunk build --release

# ── Serve stage ──
FROM nginx:alpine

COPY --from=builder /build/ats-proof-tools/dist /usr/share/nginx/html
COPY ats-proof-tools/nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
