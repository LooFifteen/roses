# syntax=docker/dockerfile:1
FROM rust:alpine AS base

ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"
WORKDIR /app

RUN apk add --no-cache mold curl

RUN rustup show && \
    cargo install cargo-chef --locked

# ---
FROM base AS planner

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

# ---
FROM base AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY assets ./assets
COPY src ./src

RUN cargo build --release

# ---
FROM scratch

COPY --from=builder /app/target/release/roses /roses

EXPOSE 3000

ENTRYPOINT [ "/roses" ]
