# Build the project
FROM rust:1.75 as build

WORKDIR /usr/src/inferno
RUN mkdir -p /usr/src/inferno
RUN apt-get update && apt-get install cmake -y

COPY . .

RUN cargo install --path ./core

# Execution does not require build tools
FROM debian:bookworm-slim as exec

WORKDIR /var/lib/inferno
COPY test-level /var/lib/inferno/test-level
COPY --from=build /usr/local/cargo/bin/inferno /usr/local/bin/inferno

CMD inferno

