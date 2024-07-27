# Use an official Rust image as a builder
FROM rust:1.75 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin gimmie_server
WORKDIR /gimmie_server

# Copy the Cargo.toml and Cargo.lock files and build dependencies
COPY . .
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code and build the application
COPY ./src ./src
RUN rm ./target/release/deps/gimmie_server*
RUN cargo build --release

# Use the same Rust base image for the final image
FROM rust:1.75
COPY --from=builder /gimmie_server/target/release/gimmie_server /usr/local/bin/gimmie_server
EXPOSE 443 80
CMD ["gimmie_server"]

