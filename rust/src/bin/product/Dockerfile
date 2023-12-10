FROM rust:1.69 as build

ADD --chown=root:root . ./
RUN cargo build --bin product_api --release

FROM debian:bullseye
COPY --from=build /target/release/product_api .

CMD ["./product_api"]