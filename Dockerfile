FROM rust:1.60 as build

RUN USER=root cargo new --bin tartar-rs
WORKDIR /tartar-rs

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

RUN rm src/*.rs
COPY ./src ./src

RUN rm ./target/release/deps/tartar-rs*
RUN cargo build --release

FROM debian:buster-slim
COPY --from=build /tartar-rs/target/release/tartar-rs .

CMD ["./tartar-rs"]