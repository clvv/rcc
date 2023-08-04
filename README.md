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

## Example circuit

An example circuit is given in `examples/circuit.rs`.

```rust
use rcc::mock_composer::{MockComposer as Composer, Wire, F, new_context_of};

const N: usize = 10;
const M: usize = 10;

#[new_context_of(e)]
// `mul_seq` is repeated `N` times in this circuit
// Encapsulates a new context to speed up compilation of witness gen code
// Try removing this and test compilation speed
fn mul_seq(e: &mut Composer, a: Wire, b: Wire) -> Wire {
    let mut v = vec![a * b];
    for i in 0..M {
        v.push(v[i] * v[i]);
    }
    v[M]
}

#[new_context_of(e)]
fn gen(e: &mut Composer, val: Wire) -> Vec<(Wire, Wire)> {
    (0..N).map(|i| {
        (
            e + i,
            e - i
        )
    }).collect()
}

pub fn my_circuit(e: &mut Composer) {
    let val = e.new_wire();
    e.arg_read(val, 1);

    let ab = gen(e, val);

    let c: Vec<Wire> = ab.iter().map(|(ai, bi)| {
        mul_seq(e, *ai, *bi)
    }).collect();

    let sum = e.sum(c);

    e.log(sum);
}
```

A functionally equivalent circuit in `circom` is given in `example_circom/example.circom`:

```circom
pragma circom 2.1.2;

template MulSeq(M) {
    signal input a;
    signal input b;

    signal c[M+1];

    signal output d;

    c[0] <== a * b;

    for (var i = 0; i < M; i++) {
        c[i+1] <== c[i] ** 2;
    }

    d <== c[M];
}

template Gen(N) {
    signal input val;
    signal output a[N];
    signal output b[N];

    for (var i = 0; i < N; i++) {
        a[i] <== val + i;
        b[i] <== val - i;
    }
}

template Main(N, M) {
    signal input val;
    signal a[N];
    signal b[N];

    (a, b) <== Gen(N)(val);

    signal c[N];
    signal output d;

    for (var i = 0; i < N; i++) {
        c[i] <== MulSeq(M)(a[i], b[i]);
    }

    var sum;
    for (var i = 0; i < N; i++) {
        sum += c[i];
    }

    d <== sum;
    log(d);
}

component main = Main(10, 10);
```

To compare performance and run the Circom circuit, run

```
cd example_circom
time circom test.circom --wasm
time node example_js/generate_witness.js example_js/example.wasm input.json outputs.wtns
```
