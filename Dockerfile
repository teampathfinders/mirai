# Build the project
FROM rust:1.75 as build

WORKDIR /usr/src/mirai
RUN mkdir -p /usr/src/mirai
RUN apt-get update && apt-get install cmake -y

COPY . .

RUN cargo install --path ./core

# Execution does not require build tools
FROM debian:bookworm-slim as exec

WORKDIR /var/lib/mirai
COPY test-level /var/lib/mirai/test-level
COPY --from=build /usr/local/cargo/bin/mirai /usr/local/bin/mirai

CMD mirai

