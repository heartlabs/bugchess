FROM rust as builder
COPY src /code/src
COPY Cargo.toml /code/Cargo.toml
COPY Cargo.lock /code/Cargo.lock
WORKDIR /code
RUN cargo build --release


WORKDIR /code/target/release
RUN chmod +x game-server
ENTRYPOINT ./game-server