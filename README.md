# Rust Circuit Compiler (RCC)

A proof-of-concept zk circuit building and witness generation library.

## Why another circuit builder?

RCC has two main differentiating factors from existing zk circuit libraries.

1. **Separation of circuit building and witness generation.** In RCC, circuit
   builders do not compute witnesses, instead, they generate
   rust code which is then compiled into the witness generator (similar to
   Circom). This makes witness generation very efficient.
2. **Abstraction via traits.** High-level circuits can be built generically
   over traits, which enables circuit re-use. Backends can implement these
   traits in the most efficient manner available to the backend, such via
   custom and lookup gates.

## FAQ

- Q: I'm building a new zk proving backend in rust, why should I integrate with
RCC?
- A:
    1. Implementing an RCC circuit builder means that your backend get access
       to high-level circuit functionalities, for free.
    2. RCC provide a way for your circuit builder to build a very efficient
       witness generator.

## RCC vs. ...

We take heavy inspirations and lessons from [Circom](https://docs.circom.io/) and
other rust-based circuit building libraries, including:
* [Arkworks R1CS builder](https://github.com/arkworks-rs/r1cs-std)
* [ECLAIR / OpenZL](https://github.com/openzklib/openzl)
* [`halo2-lib` by Axiom](https://github.com/axiom-crypto/halo2-lib)
* [`halo2-wrong` by PSE](https://github.com/privacy-scaling-explorations/halo2wrong)
* [Plonky2](https://github.com/mir-protocol/plonky2)
* [Boojum](https://github.com/matter-labs/era-boojum)

The main difference between RCC and other circuit builders in Rust:
* RCC works like Circom, where witness generation code is generated during
  circuit building. This enables parallelism for witness generation.
* RCC focuses on ergonomics of circuit writing (see [traits](rcc/src/traits)) and
  provides command-line tools to work with circuits (TODO).

The main difference between RCC and Circom:
* Witness generator in RCC is not restricted to field operations and can be
  arbitrary rust code.
* RCC is not tied to a particular proving backend. It is relatively easy to
  write a new proving backend for RCC. RCC also aims to support all style of
  proof systems, from R1CS, Plonkish, to layered arithmetic circuits (e.g. GKR).

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

To compile the demo circuit specified in `circuit-examples/examples/circuit.rs` file, `cd` into `circuit-examples` and run

```
cargo run --release --example circuit
```

To compile and run witness generation binary (generated at `examples/circuit_runtime.rs`) file with input 999, run

```
cargo run --release --example circuit_runtime 999
```

## Circuit Component

A circuit component is a function that
- Takes input a mutable reference to a composer
  - The composer exposes interfaces for circuit building
- Takes arbitrary input, including data structures over `Wire`
- Gives arbitrary outputs, including data structures over `Wire`

A circuit component can call other circuit components. However, recursive calls
are not allowed.

## Example circuits

Example circuits can be found in [`circuit-examples`](circuit-examples).

