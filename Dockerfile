# =============================================================================
# Stage 1: Build the Rust application
# =============================================================================
FROM rust:1.91-slim AS builder

WORKDIR /app

# Copy manifests first to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs so we can build dependencies first (cached layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only (this layer is cached unless Cargo.toml/Cargo.lock change)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Touch main.rs to force rebuild of the application (not dependencies)
RUN touch src/main.rs

# Build the final binary
RUN cargo build --release

# =============================================================================
# Stage 2: Create a minimal runtime image
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies (ca-certificates for HTTPS, curl for health check)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/product-twin .

# Expose the application port
EXPOSE 3000

# Set default environment variables
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
CMD ["./product-twin"]
