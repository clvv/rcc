use crate::traits::{AlgWire, BoolWire};

/// A trait indicating that a wire can be decomposed into bits
pub trait BitDecomposition: AlgWire {
    type Bool: BoolWire;

    /// Bit decompose this wire to a list of wires in big-endian.
    /// If `num_bits` is equal to the bit length of the field, then there is
    /// two possible bit decompositions that are valid.
    fn to_bits_be(&self, num_bits: usize) -> Vec<Self::Bool>;
    fn to_bits_be_strict(&self) -> Vec<Self::Bool>;
    fn to_bits_le(&self, num_bits: usize) -> Vec<Self::Bool>;
    fn to_bits_le_strict(&self) -> Vec<Self::Bool>;
    fn from_bits_be(_: Vec<Self::Bool>) -> Self;
    fn from_bits_le(_: Vec<Self::Bool>) -> Self;
}
