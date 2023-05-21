FROM rust:1.69 as build

ADD --chown=root:root . ./
RUN cargo build --bin counter_api --release

FROM debian:bullseye
COPY --from=build /target/release/counter_api .

CMD ["./counter_api"]