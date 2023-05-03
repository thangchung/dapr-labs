FROM rust:1.69 as build

RUN USER=root cargo new --bin coffeeshop
WORKDIR /coffeeshop

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src/bin/counter/main.rs ./src/bin/counter/
RUN cargo build --bin counter_api --release

RUN rm src/*.rs
COPY ./src/ ./src/

RUN rm ./target/release/deps/counter_api*
RUN cargo build --bin counter_api --release

FROM debian:buster-slim
COPY --from=build /coffeeshop/target/release/counter_api .

CMD ["./counter_api"]