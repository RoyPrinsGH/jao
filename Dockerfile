FROM rust:1.95.0-bookworm AS build

WORKDIR /app

ARG CARGO_BUILD_ARGS=""

COPY Cargo.toml Cargo.lock README.md LICENSE NOTICE rust-toolchain.toml rustfmt.toml ./
COPY src ./src

RUN cargo build --release --locked ${CARGO_BUILD_ARGS}

FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="jao"
LABEL org.opencontainers.image.description="Discover and run workspace scripts from a simple CLI"
LABEL org.opencontainers.image.source="https://github.com/RoyPrinsGH/jao"
LABEL org.opencontainers.image.licenses="Apache-2.0"

RUN apt-get update \
    && apt-get install -y --no-install-recommends bash ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/jao /usr/local/bin/jao

WORKDIR /workspace

ENTRYPOINT ["jao"]
