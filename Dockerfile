# Build the project
FROM rust:1.75 as build

# Setup source directory
WORKDIR /usr/src/pyro
RUN mkdir -p /usr/src/pyro
COPY . .

# Install cmake
RUN apt-get update && apt-get install cmake -y

RUN cargo install --path ./core
CMD pyro


# Execution does not require build tools
FROM debian:bookworm-slim as exec

WORKDIR /var/lib/pyro
COPY --from=build /usr/local/cargo/bin/pyro /usr/local/bin/pyro
COPY test-level /var/lib/pyro/test-level
CMD pyro

