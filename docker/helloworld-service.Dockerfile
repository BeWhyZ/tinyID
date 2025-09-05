# HelloWorld Service Dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY libs/ ./libs/
COPY services/helloworld/ ./services/helloworld/
COPY api/ ./api/

# Build the helloworld service
RUN cargo build --release --bin server --manifest-path services/helloworld/Cargo.toml

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/server /app/helloworld-service

# Expose HTTP port
EXPOSE 8080

# Set environment variables
ENV APP_NAME=helloworld-service
ENV APP_VERSION=0.1.0
ENV APP_HOST=0.0.0.0
ENV APP_PORT=8080
ENV APP_ENVIRONMENT=production
ENV APP_DEPENDENCIES__0__NAME=user-service
ENV APP_DEPENDENCIES__0__HOST=user-service
ENV APP_DEPENDENCIES__0__PORT=9001
ENV APP_DEPENDENCIES__0__PROTOCOL=grpc
ENV APP_TRACING__ENABLED=true
ENV APP_TRACING__SERVICE_NAME=helloworld-service
ENV APP_TRACING__SAMPLE_RATE=1.0

CMD ["./helloworld-service"]
