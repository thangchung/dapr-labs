# Component model experiment

```sh
> cargo component new command && cd command
> cargo component build
> wasmtime run --wasm-features component-model target/wasm32-wasi/debug/command.wasm 
root ➜ /workspaces/dapr-labs/component-model-expr/command (feat/component-model) $ wasmtime run --wasm-features component-model target/wasm32-wasi/debug/command.wasm 
Error: failed to run main module `target/wasm32-wasi/debug/command.wasm`

Caused by:
    0: import `docs:calculator/calculate@0.1.0` has the wrong type
    1: instance export `eval-expression` has the wrong type
    2: expected func found nothing
```

```sh
> cargo component new adder --reactor && cd adder
> cargo component build
```

```sh
> cargo component new calculator --reactor && cd calculator
> cargo compent build
```

```sh
>
> wasm-tools component wit -t composed.wasm
> wasm-tools compose spin-app/target/wasm32-wasi/release/spin_app.wasm -d composed.wasm -o spin_app.wasm
> wasm-tools component new spin_app.wasm --adapt wasi_snapshot_preview1.command.wasm -o spin_app_adapted.wasm
```

## Refs
- https://component-model.bytecodealliance.org/language-support/rust.html
- https://github.com/bytecodealliance/component-docs/blob/main/component-model/examples/tutorial/README.md