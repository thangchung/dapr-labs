FROM --platform=${BUILDPLATFORM} rust:1.67 AS build
RUN rustup target add wasm32-wasi
COPY . /product
WORKDIR /product
RUN cargo build --target wasm32-wasi --release

FROM scratch
COPY --from=build /product/target/wasm32-wasi/release/product_api.wasm /target/wasm32-wasi/release/product_api.wasm
COPY ./spin.toml /spin.toml