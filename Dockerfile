FROM rust:latest as builder

WORKDIR /usr/src/lithos

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --bin lithos

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/lithos/target/release/lithos /app/lithos

RUN useradd -m -u 1000 lithos && \
    chown -R lithos:lithos /app

USER lithos

ENV RUST_LOG=info
ENV LITHOS_TICK_DURATION_US=1
ENV LITHOS_BURST_DROPLETS=100
ENV LITHOS_SIMULATION_DURATION_MS=50

CMD ["./lithos"]