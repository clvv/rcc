# Traits

RCC provides two categories of traits: wire traits and composer traits.

* Wire traits encode types and structures over circuit wires (e.g. boolean,
  bytes, and vectors) and provide operations on them. To implement a high-level
  circuit, one only need to learn the data traits provided by RCC.
* Composer traits are traits for the circuit composer. To implement a new
  backend, one only need to implement the relevant composer traits.


## Design Philosophy

1. Functions of wire traits **should never** take `&mut Builder` as input. We want
   to keep the interfaces as clean as possible and rely on interior mutability
   of wires (all wires encode their builder).
2. We support standard Rust operators for wire traits when they make sense
   semantically.
3. When the above is not possible, we define functions with names that provide
   clear context on their semantics. e.g. `num_to_bits_be` (big endian) instead
   of `num_to_bits`.
4. Expected behavior of trait functions must be clearly documented.

