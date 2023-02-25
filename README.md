# Rust Circuit Compiler (RCC)

A proof-of-concept circuit building and witness generation library.

## Compilation Pipeline

The high-level flow is as follows:

```
                                           ┌─────────────────┐
                                       ┌──►│ circuit_config  │
                                       │   └─────────────────┘
                                       │
                                       │
┌──────────────┐ rustc   ┌───────────┐ │   ┌─────────────────┐  rustc   ┌──────────┐    ┌───────────┐
│  circuit.rs  ├─────────┤  binary   ├─┴──►│ runtime.rs      ├─────────►│ runtime  ├───►│ witnesses │
└──────────────┘         └───────────┘     └─────────────────┘          └─────▲────┘    └───────────┘
                                                                              │
                                                                              │
                                                                              │
                                                                     ┌────────┴────────┐
                                                                     │ input           │
                                                                     └─────────────────┘
```

To compile the demo circuit specified in `src/bin/circuit.rs` file, run

```
cargo --release --bin circuit
```

To compile and run witness generation binary (generated at `src/bin/circuit_runtime.rs`) file with input 3, run

```
cargo --release --bin circuit_runtime 3
```


## Circuit Component

A circuit component is a function that
- Takes input a mutable reference to a composer
  - The composer exposes interfaces for circuit building
- Takes arbitrary input, including data structures over `Wire`
- Gives arbitrary outputs, including data structures over `Wire`
A circuit component can call other circuit components. However, recursive calls
are not allowed.

