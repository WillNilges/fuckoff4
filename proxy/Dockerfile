FROM docker.io/lukemathwalker/cargo-chef AS planner
WORKDIR /usr/src/proxy
COPY Cargo.toml .
COPY src/ src/
RUN cargo chef prepare --recipe-path recipe.json

FROM docker.io/lukemathwalker/cargo-chef AS cacher
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
FROM rust:1.74 AS runtime
#RUN apt-get -y update && apt-get -y install libssl3
COPY --from=builder /usr/src/proxy/target/release/proxy /usr/local/bin/proxy
CMD ["proxy"]

