# Build the project
FROM rust:1.80 AS build

WORKDIR /usr/src/mirai
RUN mkdir -p /usr/src/mirai
RUN apt-get update && apt-get install cmake -y

COPY . .

RUN cargo install --path ./crates/core

# Execution does not require build tools
FROM debian:bookworm-slim AS exec

WORKDIR /var/lib/mirai
COPY resources resources
COPY --from=build /usr/local/cargo/bin/mirai /usr/local/bin/mirai

ENV LOG_LEVEL=DEBUG

ENTRYPOINT ["mirai"]

