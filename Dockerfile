# Argus Daemon - always-on Telegram bot + agent runtime
# Build stage
FROM rust:1.83-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

# Build release binary
RUN cargo build --release --bin argus

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies + docker CLI (for workspace exec)
RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 docker.io \
    && rm -rf /var/lib/apt/lists/*

# Create argus user and add to docker group
RUN groupadd -g 1000 argus && \
    useradd -u 1000 -g argus -m argus && \
    usermod -aG docker argus

# Create data directory
RUN mkdir -p /argus/data && chown -R argus:argus /argus

WORKDIR /argus
USER argus

# Copy binary from builder
COPY --from=builder /build/target/release/argus /usr/local/bin/argus

# Persistent storage for memory
VOLUME ["/argus/data"]

# Health check - just verify the binary runs
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD argus --version || exit 1

# Run daemon mode by default
CMD ["argus", "daemon"]
