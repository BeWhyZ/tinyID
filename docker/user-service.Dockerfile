# User Service Dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY libs/ ./libs/
COPY services/user/ ./services/user/
COPY api/ ./api/

# Build the user service
RUN cargo build --release --bin server --manifest-path services/user/Cargo.toml

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/server /app/user-service

# Expose gRPC port
EXPOSE 9001

# Set environment variables
ENV APP_NAME=user-service
ENV APP_VERSION=0.1.0
ENV APP_HOST=0.0.0.0
ENV APP_PORT=8081
ENV APP_GRPC_PORT=9001
ENV APP_ENVIRONMENT=production
ENV APP_TRACING__ENABLED=true
ENV APP_TRACING__SERVICE_NAME=user-service
ENV APP_TRACING__SAMPLE_RATE=1.0

CMD ["./user-service"]
