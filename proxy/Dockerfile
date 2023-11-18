# STAGE 1: Build
FROM rust:1.74 as builder

WORKDIR /usr/src/
RUN cargo init proxy
WORKDIR /usr/src/proxy
COPY Cargo.toml .
RUN cargo build --release
COPY src/ src/

RUN cargo install --path .

# STAGE 2: Shipit lol
FROM debian:bullseye-slim

#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp

CMD ["proxy"]


FROM lukemathwalker/cargo-chef AS planner
WORKDIR /usr/src/proxy
COPY Cargo.toml .
COPY src/ src/
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef AS cacher
WORKDIR /usr/src/proxy
COPY --from=planner /usr/src/proxy/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust AS builder
WORKDIR /usr/src/proxy
COPY Cargo.toml .
COPY src/ src/
# Copy over the cached dependencies
COPY --from=cacher /usr/src/proxy/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --bin proxy

# STAGE 2: Shipit lol
FROM debian:bullseye-slim AS runtime
COPY --from=builder /usr/src/proxy/target/release/myapp /usr/local/bin/myapp
CMD ["proxy"]

