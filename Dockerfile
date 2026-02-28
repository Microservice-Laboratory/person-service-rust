# Using cargo-chef for better caching of dependencies
FROM rust:1.88-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 1: Plan the build
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Layer 2: Build the dependencies and the application
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Install build dependencies required by rdkafka and openssl
RUN apt-get update && apt-get install -y \
    cmake \
    build-essential \
    pkg-config \
    libssl-dev \
    libcurl4-openssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies (this layer is cached)
RUN cargo chef cook --release --recipe-path recipe.json

# Build the application
COPY . .
RUN cargo build --release

# Stage 3: Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    libcurl4 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/person-service-rust /app/person-service-rust

# Set runtime environment variables
ENV DATABASE_URL=""

# Expose the application port
EXPOSE 3000

# Start the application
CMD ["/app/person-service-rust"]
