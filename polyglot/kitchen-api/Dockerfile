FROM --platform=${BUILDPLATFORM} rust:1.67 AS build
RUN rustup target add wasm32-wasi
COPY . /kitchen
WORKDIR /kitchen
RUN cargo build --target wasm32-wasi --release

FROM scratch
COPY --from=build /kitchen/target/wasm32-wasi/release/kitchen_api.wasm /target/wasm32-wasi/release/kitchen_api.wasm
COPY ./spin.toml /spin.toml
ENTRYPOINT ["/"]