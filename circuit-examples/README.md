# RCC Circuit Examples

## Halo2 Example

An example circuit for a simple Halo2 backend is provided in
[`examples/halo2_example.rs`](examples/halo2_example.rs).

To compile the circuit, run

```
time cargo run --release --example mock_example
```

This generates a runtime library file `example/halo2_example_runtime_lib.rs` as
well as a circuit config file `example/halo2_example_config.toml`.

To run mock proof generation with input `999`, run

```
time cargo run --release --example mock_example_runtime 999
```

## Witness generator speed test

An example circuit is given in [`examples/mock_example.rs`](examples/mock_example.rs).

To compile the `mock_example` circuit, `cd` into `circuit-examples` and run

```
time cargo run --release --example mock_example
```

This will generate the runtime library file for the circuit in
`examples/mock_example_runtime_lib.rs`. To run the witness generator with input
set to `999`, run

```
time cargo run --release --example mock_example_runtime 999
```

A Circom circuit of equivalent functionality is given in
[`example-circom/example.cirom`](example-circom/example.circom). For
comparison, the Circom circuit can be compiled and run via the following
commands inside `example_circom`.

```
time circom example.circom --wasm
time node example_js/generate_witness.js example_js/example.wasm input.json outputs.wtns
```

