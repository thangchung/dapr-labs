FROM rust:1.69 as build

ADD --chown=root:root . ./
RUN cargo build --bin barista_api --release

FROM debian:bullseye
RUN apt-get update && apt-get install -y libssl-dev libcurl4
COPY --from=build /target/release/barista_api .

CMD ["./barista_api"]