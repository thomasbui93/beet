# --- Stage 1: Build the binary ---
FROM rust:1.85-slim AS builder

WORKDIR /usr/src/app

# Copy the entire source tree
COPY . .

# Build the release artifact
RUN cargo build --release

# --- Stage 2: Minimal Runtime Environment ---
FROM debian:bookworm-slim

WORKDIR /app

# Copy only the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/beet ./beet
COPY .env ./

EXPOSE 8080

# Run the server
CMD ./beet