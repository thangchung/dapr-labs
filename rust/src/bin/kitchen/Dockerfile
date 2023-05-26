FROM rust:1.69 as build

ADD --chown=root:root . ./
RUN cargo build --bin kitchen_api --release

FROM debian:bullseye
RUN apt-get update && apt-get install -y libssl-dev libcurl4
COPY --from=build /target/release/kitchen_api .

CMD ["./kitchen_api"]