use crate::{
    traits::{AlgWire, BoolWire, ToBits},
    WithGlobalBuilder
};

use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Sub, Div, Shl, Shr};

use super::Boolean;

/// Trait for an unsigned integer of arbitrary bitlength
pub trait UInt32:
    Add<Output = Self>
    + Add<u32, Output = Self>
    + Sub<Output = Self>
    + Sub<u32, Output = Self>
    + Mul<u32, Output = Self>
    + Mul<Output = Self>
    + Div<u32, Output = Self>
    + Div<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + BitAnd<u32, Output = Self>
    + BitAnd<Output = Self>
    + BitOr<u32, Output = Self>
    + BitOr<Output = Self>
    + BitXor<u32, Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Sized
    + Copy
{
    type DenseRepr;

    fn from_const(_: u32) -> Self;

    fn num_bits(&self) -> usize;
    fn rotate_right(&self, c: u32) -> Self {
        assert!(c < 32, "Cannot rotate by more than 31 bits");
        (*self >> c) ^ (*self << (32 - c))
    }
    fn to_dense(&self) -> Self::DenseRepr;
}

/// An naive implementaiton of UInt32 using boolean wires
#[derive(Copy, Clone)]
pub struct NaiveUInt32<Bool: BoolWire> {
    pub repr: [Bool; 32]
}

impl<T: AlgWire + WithGlobalBuilder> WithGlobalBuilder for NaiveUInt32<Boolean<T>> {
    type Builder = <T as WithGlobalBuilder>::Builder;

    fn global_builder() -> &'static mut <Self as WithGlobalBuilder>::Builder {
        T::global_builder()
    }
}

impl<T: AlgWire + ToBits> NaiveUInt32<Boolean<T>> {
    pub fn from_vec(repr_vec: Vec<Boolean<T>>) -> Self {
        NaiveUInt32 { repr: repr_vec.try_into().unwrap_or_else(|_| unreachable!()) }
    }
}

impl<T: AlgWire + ToBits<Bool = Boolean<T>>> UInt32 for NaiveUInt32<Boolean<T>> {
    type DenseRepr = T;

    fn num_bits(&self) -> usize {
        self.repr.len()
    }

    fn from_const(c: u32) -> Self {
        let repr_vec: Vec<_> = (0..32).map(|i| {
            Boolean::<T>::from_const((c >> (31 - i)) & 1)
        }).collect();
        NaiveUInt32::from_vec(repr_vec)
    }

    fn rotate_right(&self, c: u32) -> Self {
        let repr_vec: Vec<_> = (0..32).map(|i| {
            if i >= c {
                self.repr[(i - c) as usize]
            } else {
                self.repr[(i - c + 32) as usize]
            }
        }).collect();

        NaiveUInt32::from_vec(repr_vec)
    }

    fn to_dense(&self) -> T {
        T::from_bits_be(self.repr.into())
    }
}

impl<T: AlgWire + ToBits<Bool = Boolean<T>>> Add for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut accum_bits = (0..32).map(|i| {
            (self.repr[i].to_alg() + other.repr[i].to_alg()) * (1u32 << (31 - i))
        });

        let mut sum = accum_bits.next().unwrap();
        for a in accum_bits {
            sum = sum + a
        }

        // Decompose the sum
        let mut bits = sum.to_bits_be(33);
        bits.remove(0);
        NaiveUInt32::from_vec(bits)
    }
}

impl<C, T: AlgWire + ToBits<Bool = Boolean<T>>> Add<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn add(self, other: C) -> Self {
        self + Self::from_const(other.into())
    }
}

impl<T: AlgWire + ToBits> Sub for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn sub(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire + ToBits> Sub<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn sub(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire + ToBits> Mul for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn mul(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire + ToBits> Mul<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn mul(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire + ToBits> Div for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn div(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire + ToBits> Div<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn div(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire + ToBits> BitAnd for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        let repr_vec: Vec<_> = (0..32).map(|i| {
            self.repr[i] & other.repr[i]
        }).collect();

        NaiveUInt32::from_vec(repr_vec)
    }
}

impl<C, T: AlgWire + ToBits> BitAnd<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn bitand(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire + ToBits> BitOr for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        let repr_vec: Vec<_> = (0..32).map(|i| {
            self.repr[i] | other.repr[i]
        }).collect();

        NaiveUInt32::from_vec(repr_vec)
    }
}

impl<C, T: AlgWire + ToBits> BitOr<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn bitor(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire + ToBits> BitXor for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        let repr_vec: Vec<_> = (0..32).map(|i| {
            self.repr[i] ^ other.repr[i]
        }).collect();

        NaiveUInt32::from_vec(repr_vec)
    }
}

impl<C, T: AlgWire + ToBits> BitXor<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn bitxor(self, _other: C) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire + ToBits> Shl<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn shl(self, _other: C) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire + ToBits> Shr<C> for NaiveUInt32<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn shr(self, c: C) -> Self {
        let c: u32 = c.into();
        let repr_vec: Vec<_> = (0..32).map(|i| {
            if i >= c {
                self.repr[(i - c) as usize]
            } else {
                Boolean::<T>::from_const(0)
            }
        }).collect();

        NaiveUInt32::from_vec(repr_vec)
    }
}

impl<T: AlgWire + ToBits> Not for NaiveUInt32<Boolean<T>> {
    type Output = Self;

    fn not(self) -> Self {
        let repr_vec: Vec<_> = (0..32).map(|i| {
            !self.repr[i]
        }).collect();

        NaiveUInt32::from_vec(repr_vec)
    }
}
