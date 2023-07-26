pub use crate::{Composer, Wire};
use quote::{quote, ToTokens};

use std::ops::{Add, Sub, Mul, Neg, BitAnd, BitOr, BitXor, Not};

pub trait ALWire:
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
    fn inv_unchecked(self, default: u32) -> Self {
        let e = self.composer();
        let w_inv = e.new_wire();
        e.runtime(quote! {
            #w_inv = #self.inverse().unwrap_or(#default.into());
        });
        w_inv
    }

    fn assert_not_zero(self) {
        let w_inv = self.inv_unchecked(0);
        self * w_inv == 1;
    }

    /// Maps any non-zero field element to one and zero to zero.
    fn to_bool(self) -> Bool<Self> {
        let w_inv = self.inv_unchecked(1);
        w_inv.assert_not_zero();
        Bool { wire: self * w_inv }
    }

    /// Assert that the wire is boolean
    fn check_bool(self) -> Bool<Self> {
        self * (self - 1) == 0;
        Bool { wire: self }
    }
}

/// Struct to capture a boolean wire
#[derive(Debug, Copy, Clone)]
pub struct Bool<T: ALWire> {
    pub wire: T
}

impl<T: ALWire> Bool<T> {
    pub fn then(self, then: T) -> BoolThen<T> {
        BoolThen {
            wire: self.wire,
            then,
        }
    }
}

pub struct BoolThen<T: ALWire> {
    wire: T,
    then: T,
}

impl<T: ALWire> BoolThen<T> {
    pub fn els(self, other: T) -> T {
        self.wire * self.then + (-self.wire + 1) * other
    }
}

impl<T: ALWire> BitAnd<Bool<T>> for Bool<T> {
    type Output = Bool<T>;
    fn bitand(self, rhs: Bool<T>) -> Bool<T> {
        Bool { wire: self.wire * rhs.wire }
    }
}

impl<T: ALWire> BitOr<Bool<T>> for Bool<T> {
    type Output = Bool<T>;
    fn bitor(self, rhs: Bool<T>) -> Bool<T> {
        Bool { wire: self.wire + rhs.wire - self.wire * rhs.wire }
    }
}

impl<T: ALWire> BitXor<Bool<T>> for Bool<T> {
    type Output = Bool<T>;
    fn bitxor(self, rhs: Bool<T>) -> Bool<T> {
        Bool { wire: self.wire + rhs.wire - self.wire * rhs.wire * 2 }
    }
}

impl<T: ALWire> Not for Bool<T> {
    type Output = Bool<T>;
    fn not(self) -> Bool<T> {
        Bool { wire: -self.wire + 1 }
    }
}
