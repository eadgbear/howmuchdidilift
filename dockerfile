FROM rust:1.94-slim-bookworm as builder

# C toolchain for crates with native build steps (ring/rustls, etc.).
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app

COPY --from=builder /usr/src/frontend/dist /usr/app/frontend/dist
COPY --from=builder /usr/src/frontend/dist/index.html /usr/app/frontend/dist/index.html
COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/assets /usr/app/assets
COPY --from=builder /usr/src/target/release/liftcalc-cli /usr/app/liftcalc-cli

ENTRYPOINT ["/usr/app/liftcalc-cli"]

CMD ["start", "-e", "production"]