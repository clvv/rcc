pub use crate::{Composer, Wire};
use quote::{quote, ToTokens};

use std::ops::{Add, Sub, Mul, Neg, BitAnd, BitOr, BitXor, Not};

/// Trait for arithemtic logic wires
pub trait AlgWire:
    Add<Output = Self> +
    // Add<u8, Output = Self> +
    // Add<u16, Output = Self> +
    Add<u32, Output = Self> +
    Add<u64, Output = Self> +
    // Add<i8, Output = Self> +
    // Add<i16, Output = Self> +
    Add<i32, Output = Self> +
    Add<i64, Output = Self> +
    Sub<Output = Self> +
    // Sub<u8, Output = Self> +
    // Sub<u16, Output = Self> +
    Sub<u32, Output = Self> +
    Sub<u64, Output = Self> +
    // Sub<i8, Output = Self> +
    // Sub<i16, Output = Self> +
    Sub<i32, Output = Self> +
    Sub<i64, Output = Self> +
    // Sub<F, Output = Self> +
    Mul<Output = Self> +
    // Mul<u8, Output = Self> +
    // Mul<u16, Output = Self> +
    Mul<u32, Output = Self> +
    Mul<u64, Output = Self> +
    // Mul<i8, Output = Self> +
    // Mul<i16, Output = Self> +
    Mul<i32, Output = Self> +
    Mul<i64, Output = Self> +
    // Mul<F, Output = Self> +
    PartialEq +
    // PartialEq<u8> +
    // PartialEq<u16> +
    PartialEq<u32> +
    PartialEq<u64> +
    // PartialEq<i8> +
    // PartialEq<i16> +
    PartialEq<i32> +
    PartialEq<i64> +
    // PartialEq<F> +
    Neg<Output = Self> +
    ToTokens +
    Sized + Copy + Wire
{
    type Bool: BoolWire;

    fn inv_or_default(self, default: u32) -> Self {
        let e = self.composer();
        let w_inv = e.new_wire();
        e.runtime(quote! {
            #w_inv = #self.inverse().unwrap_or(#default.into());
        });
        w_inv
    }

    fn assert_not_zero(self) {
        let w_inv = self.inv_or_default(0);
        self * w_inv == 1;
    }

    /// Maps any non-zero field element to one and zero to zero.
    fn to_bool(self) -> Self::Bool;

    /// Assert that the wire is boolean
    fn check_bool(self) -> Self::Bool;
}

/// Trait for a wire holding a boolean value
pub trait BoolWire:
    BitAnd<Output = Self> +
    BitOr<Output = Self> +
    BitXor<Output = Self> +
    Not<Output = Self> + Sized + Copy
{
    type AlgWire;

    fn then_or_else(&self, then: Self::AlgWire, els: Self::AlgWire) -> Self::AlgWire;
}


#[derive(Debug, Copy, Clone)]
/// Default implementation of a boolean wire, generic over any AlgWire
pub struct Boolean<T: AlgWire> {
    pub wire: T
}

impl<T: AlgWire> BitAnd<Boolean<T>> for Boolean<T> {
    type Output = Boolean<T>;
    fn bitand(self, rhs: Boolean<T>) -> Boolean<T> {
        Boolean { wire: self.wire * rhs.wire }
    }
}

impl<T: AlgWire> BitOr<Boolean<T>> for Boolean<T> {
    type Output = Boolean<T>;
    fn bitor(self, rhs: Boolean<T>) -> Boolean<T> {
        Boolean { wire: self.wire + rhs.wire - self.wire * rhs.wire }
    }
}

impl<T: AlgWire> BitXor<Boolean<T>> for Boolean<T> {
    type Output = Boolean<T>;
    fn bitxor(self, rhs: Boolean<T>) -> Boolean<T> {
        Boolean { wire: self.wire + rhs.wire - self.wire * rhs.wire * 2 }
    }
}

impl<T: AlgWire> Not for Boolean<T> {
    type Output = Boolean<T>;
    fn not(self) -> Boolean<T> {
        Boolean { wire: -self.wire + 1 }
    }
}

impl<T: AlgWire> BoolWire for Boolean<T> {
    type AlgWire = T;

    fn then_or_else(&self, then: T, els: T) -> T {
        self.wire * then + (-self.wire + 1) * els
    }
}
