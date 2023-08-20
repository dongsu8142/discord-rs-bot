FROM rust:slim AS chef

WORKDIR /usr/src/project

RUN apt-get update && apt-get install -y pkg-config libssl-dev libopus-dev
RUN set -eux; \
    cargo install cargo-chef; \
    rm -rf $CARGO_HOME/registry

FROM chef as planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/project/recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM python:slim-bullseye

WORKDIR /usr/local/bin

RUN apt-get update && apt-get install -y libopus-dev
RUN python3 -m pip install -U yt-dlp

COPY --from=builder /usr/src/project/target/release/discord-rs .

CMD ["discord-rs"]
