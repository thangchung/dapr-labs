FROM --platform=${BUILDPLATFORM} rust:1.67 AS build
RUN rustup target add wasm32-wasi
COPY . /barista
WORKDIR /barista
RUN cargo build --target wasm32-wasi --release

FROM scratch
COPY --from=build /barista/target/wasm32-wasi/release/barista_api.wasm /target/wasm32-wasi/release/barista_api.wasm
COPY ./spin.toml /spin.toml
ENTRYPOINT ["/"]