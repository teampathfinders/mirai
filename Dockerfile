# Build the project
FROM rust:1.75 as build

WORKDIR /usr/src/pyro
RUN mkdir -p /usr/src/pyro
RUN apt-get update && apt-get install cmake -y

COPY . .

RUN cargo install --path ./core

# Execution does not require build tools
FROM debian:bookworm-slim as exec

WORKDIR /var/lib/pyro
COPY test-level /var/lib/pyro/test-level
COPY --from=build /usr/local/cargo/bin/pyro /usr/local/bin/pyro

CMD pyro

