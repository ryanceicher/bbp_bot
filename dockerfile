# Start from a Rust base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/bbp_bot

# Copy over your Manifest files
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy your source code
COPY ./src ./src

# Build for release. 
RUN cargo build --release

# # Our new stage will start from a more minimal image
# # which results in a smaller final image size
FROM debian:bullseye-slim

# Install OpenSSL
RUN apt-get update && apt-get install -y openssl

# # Copy the binary from the builder stage to the new stage
COPY --from=builder /usr/src/bbp_bot/target/release/bbp_bot /usr/local/bin

# # Run the binary
CMD ["bbp_bot"]