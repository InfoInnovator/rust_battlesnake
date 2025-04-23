FROM rust:1.85

COPY battlesnakes /usr/app/battlesnakes
COPY rocket-server /usr/app/rocket-server
COPY runa /usr/app/runa
COPY xtask /usr/app/xtask
COPY Cargo.toml /usr/app/Cargo.toml
COPY Cargo.lock /usr/app/Cargo.lock

WORKDIR /usr/app
RUN cargo build --release

CMD ["./target/release/rocket-server"]