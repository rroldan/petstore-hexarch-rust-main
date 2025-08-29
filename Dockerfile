# Builder stage
FROM rust:1.87.0 as builder

WORKDIR /usr/src/app
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install OpenSSL and CA certificates
RUN apt-get update && \
    apt-get install -y openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/petstore-hexarch-rust /app/
# Copy migrations
COPY --from=builder /usr/src/app/migrations /app/migrations

# Set environment variables
ENV RUST_LOG=info

# Expose the port
EXPOSE 3000

# Run the binary
CMD ["./"] 

#Command Run
#docker run --name postgres-app -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=postgres -p 5433:5432 --network app-network -d postgres:15