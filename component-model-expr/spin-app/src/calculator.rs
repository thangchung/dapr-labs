wasmtime::component::bindgen!({
    world: "calculator",
    path: "../wit",
    async: true,
});

// cargo_component_bindings::generate!();