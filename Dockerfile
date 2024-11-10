##### BUILDER #####
FROM rust:latest as builder

WORKDIR /usr/src/orb
COPY . .
RUN cargo install --path .

##### RUNNER #####
FROM debian:bookworm

LABEL author="Lola Rigaut-Luczak <me@laflemme.lol>"

COPY --from=builder /usr/local/cargo/bin/orb /usr/local/bin/orb

RUN apt-get update && rm -rf /var/lib/apt/lists/*

CMD orb