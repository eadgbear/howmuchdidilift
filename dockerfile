FROM rust:1.94-slim-bookworm as builder

# C toolchain for crates with native build steps (ring/rustls, etc.) + curl.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Frontend toolchain: wasm target + Trunk (prebuilt binary for the build arch).
ARG TRUNK_VERSION=v0.21.14
ARG TARGETARCH
RUN rustup target add wasm32-unknown-unknown \
    && case "${TARGETARCH}" in \
         amd64) TRUNK_ARCH=x86_64-unknown-linux-gnu ;; \
         arm64) TRUNK_ARCH=aarch64-unknown-linux-gnu ;; \
         *) echo "unsupported TARGETARCH=${TARGETARCH}" >&2; exit 1 ;; \
       esac \
    && curl -sSL "https://github.com/trunk-rs/trunk/releases/download/${TRUNK_VERSION}/trunk-${TRUNK_ARCH}.tar.gz" \
       | tar -xz -C /usr/local/bin trunk

WORKDIR /usr/src/

COPY . .

# Build the WASM frontend (produces frontend/dist) then the backend binary.
RUN cd frontend && trunk build --release
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app

COPY --from=builder /usr/src/frontend/dist /usr/app/frontend/dist
COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/assets /usr/app/assets
COPY --from=builder /usr/src/target/release/liftcalc-cli /usr/app/liftcalc-cli

ENTRYPOINT ["/usr/app/liftcalc-cli"]

CMD ["start", "-e", "production"]