FROM rust as builder
COPY src /code/src
COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
WORKDIR /code
RUN cargo build --release


WORKDIR /code/target/release


FROM debian:buster-slim
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3030

COPY --from=builder /code/target/release/game-server ./game-server
RUN chmod +x game-server
ENTRYPOINT ./game-server