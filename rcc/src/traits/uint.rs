use crate::{
    traits::{AlgBuilder, AlgWire, BoolWire},
    WireLike
};

use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Sub, Div, Shl, Shr};

use super::Boolean;

/// A trait indicating that the builder supports decomposing a wire into limbs
pub trait UIntBuilder: AlgBuilder {
    type UInt: WireLike;
    type DenseRepr: WireLike;

    fn to_dense(&self) -> Self::DenseRepr;

    fn add(&mut self, _: Self::UInt, _: Self::UInt);
    fn sub(&mut self, _: Self::UInt, _: Self::UInt);
    fn mul(&mut self, _: Self::UInt, _: Self::UInt);
    fn div(&mut self, _: Self::UInt, _: Self::UInt);
    fn and(&mut self, _: Self::UInt, _: Self::UInt);
    fn or(&mut self, _: Self::UInt, _: Self::UInt);
    fn xor(&mut self, _: Self::UInt, _: Self::UInt);
    fn neg(&mut self, _: Self::UInt);
    fn shl_const(&mut self, _: Self::UInt, _: usize);
    fn shr_const(&mut self, _: Self::UInt, _: usize);
}

/// Trait for an unsigned integer of arbitrary bitlength
pub trait UInt:
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

    fn num_bits(&self) -> usize;
    fn rotate_right(&self, c: u32) -> Self {
        assert!(c < 32, "Cannot rotate by more than 31 bits");
        (*self >> c) ^ (*self << (32 - c))
    }
    fn to_dense(&self) -> Self::DenseRepr;
}

#[derive(Copy, Clone)]
pub struct NaiveUInt<Bool: BoolWire> {
    repr: [Bool; 32]
}

impl<T: AlgWire> UInt for NaiveUInt<Boolean<T>> {
    type DenseRepr = T;

    fn num_bits(&self) -> usize {
        self.repr.len()
    }

    fn to_dense(&self) -> T {
        todo!()
    }
}

impl<T: AlgWire> Add for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn add(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> Add<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn add(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> Sub for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn sub(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> Sub<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn sub(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> Mul for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn mul(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> Mul<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn mul(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> Div for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn div(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> Div<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn div(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> BitAnd for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn bitand(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> BitAnd<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn bitand(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> BitOr for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn bitor(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> BitOr<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn bitor(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> BitXor for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn bitxor(self, _other: Self) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> BitXor<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn bitxor(self, _other: C) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> Shl<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn shl(self, _other: C) -> Self {
        todo!()
    }
}

impl<C, T: AlgWire> Shr<C> for NaiveUInt<Boolean<T>>
where
    C: Into<u32>,
{
    type Output = Self;

    fn shr(self, _other: C) -> Self {
        todo!()
    }
}

impl<T: AlgWire> Not for NaiveUInt<Boolean<T>> {
    type Output = Self;

    fn not(self) -> Self {
        todo!()
    }
}
