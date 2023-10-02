use crate::{
    traits::{AlgBuilder, AlgWire, BoolWire},
    Builder,
};

/// A trait indicating that the builder supports decomposing a wire into bits
pub trait ToBitsBuilder: AlgBuilder {
    /// Maximum num of bits required to represent the underlying field element, i.e. NUM_BITS is
    /// the smallest integer such that 2**NUM_BITS - 1 >= p.
    const NUM_BITS: usize;

    /// Bit decompose this wire to a list of wires in big-endian order.
    /// If `num_bits` is equal to the bit length of the field, then there is
    /// two possible bit decompositions that are valid.
    fn to_bits_be(&mut self, w: <Self as Builder>::Wire, num_bits: usize) -> Vec<Self::Bool>;

    /// Bit decompose this wire to a list of wires in big-endian order. The list is of length
    /// BITLENGTH, and value represented is strictly less than the modulus
    fn to_bits_be_strict(&mut self, w: <Self as Builder>::Wire) -> Vec<Self::Bool>;

    fn to_bits_le(&mut self, w: <Self as Builder>::Wire, num_bits: usize) -> Vec<Self::Bool>;
    fn to_bits_le_strict(&mut self, w: <Self as Builder>::Wire) -> Vec<Self::Bool>;
    fn from_bits_be(&mut self, _: Vec<Self::Bool>) -> Self::Wire;
    fn from_bits_le(&mut self, _: Vec<Self::Bool>) -> Self::Wire;
}

/// A trait indicating that a wire can be decomposed into bits.
pub trait ToBits: AlgWire {
    type Bool: BoolWire;

    /// Maximum num of bits required to represent the underlying field element, i.e. NUM_BITS is
    /// the smallest integer such that 2**NUM_BITS - 1 >= p.
    const NUM_BITS: usize;

    /// Bit decompose this wire to a list of wires in big-endian order.
    /// If `num_bits` is equal to the bit length of the field, then there is
    /// two possible bit decompositions that are valid.
    fn to_bits_be(self, num_bits: usize) -> Vec<Self::Bool>;

    /// Bit decompose this wire to a list of wires in big-endian order. The list is of length
    /// BITLENGTH, and value represented is strictly less than the modulus
    fn to_bits_be_strict(self) -> Vec<Self::Bool>;

    fn to_bits_le(self, num_bits: usize) -> Vec<Self::Bool>;
    fn to_bits_le_strict(self) -> Vec<Self::Bool>;

    fn from_bits_be(_: Vec<Self::Bool>) -> Self;
    fn from_bits_le(_: Vec<Self::Bool>) -> Self;
}

#[macro_export]
/// Automatically implements AlgWire trait for AlgBuilder::Wire
macro_rules! impl_to_bits {
    ($builder:ident, $wire:ident) => {
        use rcc::WithGlobalBuilder;
        impl ToBits for $wire {
            type Bool = <$builder as AlgBuilder>::Bool;
            const NUM_BITS: usize = $builder::NUM_BITS;

            fn to_bits_be(self, num_bits: usize) -> Vec<Self::Bool> {
                self.builder().to_bits_be(self, num_bits)
            }

            fn to_bits_be_strict(self) -> Vec<Self::Bool> {
                self.builder().to_bits_be_strict(self)
            }

            fn to_bits_le(self, num_bits: usize) -> Vec<Self::Bool> {
                self.builder().to_bits_le(self, num_bits)
            }

            fn to_bits_le_strict(self) -> Vec<Self::Bool> {
                self.builder().to_bits_le_strict(self)
            }

            fn from_bits_be(bits: Vec<Self::Bool>) -> Self {
                $wire::global_builder().from_bits_be(bits)
            }

            fn from_bits_le(bits: Vec<Self::Bool>) -> Self {
                $wire::global_builder().from_bits_le(bits)
            }
        }
    };
}
