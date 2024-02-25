# Build the project
FROM rust:1.75 as build

WORKDIR /usr/src/mirai
RUN mkdir -p /usr/src/mirai
RUN apt-get update && apt-get install cmake -y

COPY . .

RUN cargo install --path ./crates/core

# Execution does not require build tools
FROM debian:bookworm-slim as exec

WORKDIR /var/lib/mirai
COPY resources /var/lib/mirai/resources
COPY --from=build /usr/local/cargo/bin/mirai /usr/local/bin/mirai

CMD mirai

