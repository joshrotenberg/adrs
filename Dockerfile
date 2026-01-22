# Build stage
FROM rust:bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/adrs-core/Cargo.toml crates/adrs-core/
COPY crates/adrs/Cargo.toml crates/adrs/

# Create dummy source files to build dependencies
RUN mkdir -p crates/adrs-core/src crates/adrs/src && \
    echo "pub fn dummy() {}" > crates/adrs-core/src/lib.rs && \
    echo "fn main() {}" > crates/adrs/src/main.rs

# Build dependencies only
RUN cargo build --release --package adrs

# Remove dummy source files
RUN rm -rf crates/adrs-core/src crates/adrs/src

# Copy actual source code
COPY crates/adrs-core/src crates/adrs-core/src
COPY crates/adrs/src crates/adrs/src

# Touch files to ensure rebuild
RUN touch crates/adrs-core/src/lib.rs crates/adrs/src/main.rs

# Build the actual binary
RUN cargo build --release --package adrs

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies and create non-root user
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 adrs

# Copy binary from builder
COPY --from=builder /build/target/release/adrs /usr/local/bin/adrs

# Set up working directory
WORKDIR /workspace

# Switch to non-root user
USER adrs

ENTRYPOINT ["adrs"]
CMD ["--help"]
