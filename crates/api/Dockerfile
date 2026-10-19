# Dockerfile for agrocore-api
# Uses latest stable Rust on latest Debian (trixie)
FROM rust:latest AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Set DATABASE_URL for sqlx offline mode
ENV DATABASE_URL=postgresql://user:pass@localhost:5432/db
ENV SQLX_OFFLINE=true

# Build the API binary
RUN cargo build --release -p agrocore-api

# Runtime stage - latest Debian stable (trixie 13) to match rust:latest builder
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/agrocore-api /usr/local/bin/agrocore-api

# Create logs directory
RUN mkdir -p /app/logs && chown -R 1000:1000 /app/logs

# Copy config
COPY config /app/config

EXPOSE 8080

CMD ["agrocore-api"]