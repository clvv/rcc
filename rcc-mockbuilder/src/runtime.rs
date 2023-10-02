pub use ark_ff::{BigInt, BigInteger, Field, PrimeField};
pub use ark_bn254::Fr as F;
pub use num_bigint::BigUint;

// runtime composer expects WireVal to be defined
pub type WireVal = F;
pub type AllWires = Vec<F>;
