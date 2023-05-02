FROM rust:1.69 as build

RUN USER=root cargo new --bin coffeeshop
WORKDIR /coffeeshop

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src/bin/product/main.rs ./src/bin/product/
RUN cargo build --bin product_api --release

RUN rm src/*.rs
COPY ./src/ ./src/

RUN rm ./target/release/deps/product_api*
RUN cargo build --bin product_api --release

FROM debian:buster-slim
COPY --from=build /coffeeshop/target/release/product_api .

CMD ["./product_api"]