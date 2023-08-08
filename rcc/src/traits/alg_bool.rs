pub use crate::{Composer, Wire};
use rcc_macro::component_of;

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
    Wire
{
    fn inv_or_panic(self);
    fn inv_or_any(self);
}

pub trait AlgComposer: Composer {
    type Constant: From<i32> + From<i64> + From<u32> + From<u64>;
    type Bool: BoolWire;

    fn add(&mut self, a: Self::Wire, b: Self::Wire) -> Self::Wire;
    fn add_const(&mut self, a: Self::Wire, b: Self::Constant) -> Self::Wire;
    fn sub(&mut self, a: Self::Wire, b: Self::Wire) -> Self::Wire;
    fn sub_const(&mut self, a: Self::Wire, b: Self::Constant) -> Self::Wire;
    fn mul(&mut self, a: Self::Wire, b: Self::Wire) -> Self::Wire;
    fn mul_const(&mut self, a: Self::Wire, b: Self::Constant) -> Self::Wire;
    fn assert_eq(&mut self, a: Self::Wire, b: Self::Wire);
    fn assert_eq_const(&mut self, a: Self::Wire, b: Self::Constant);
    fn inv_or_panic(&mut self, a: Self::Wire) -> Self::Wire;
    fn inv_or_any(&mut self, a: Self::Wire) -> Self::Wire;

    /// Maps any non-zero field element to one and zero to zero.
    fn to_bool(&mut self, a: Self::Wire) -> Self::Bool;
    /// Assert that the wire is boolean
    fn assert_bool(&mut self, a: Self::Wire) -> Self::Bool;

    // Below functions has default implementations that require two "sub constraints".
    // Backends can decide to provide more efficient variants.
    fn assert_ne_const(&mut self, a: Self::Wire, b: Self::Constant) {
        let c = self.sub_const(a, b);
        self.inv_or_panic(c);
    }
    fn assert_ne(&mut self, a: Self::Wire, b: Self::Wire) {
        let c = self.sub(a, b);
        self.inv_or_panic(c);
    }

    #[component_of(self)]
    fn sum(&mut self, wires: Vec<Self::Wire>) -> Self::Wire {
        let mut running_sum = wires[0];
        self.smart_map(wires.iter(), |e, &&w| {
            running_sum = e.add(running_sum, w);
        });
        running_sum
    }

    #[component_of(self)]
    fn prod(&mut self, wires: Vec<Self::Wire>) -> Self::Wire {
        let mut running_prod = wires[0];
        self.smart_map(wires.iter(), |e, &&w| {
            running_prod = e.mul(running_prod, w);
        });
        running_prod
    }
}

#[macro_export]
/// Automatically implements AlgWire trait for AlgComposer::Wire
macro_rules! impl_alg_op {
    ($wire:ident, $constant_type:ty) => {
        impl Add for $wire {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                self.composer().add(self, other)
            }
        }

        impl<T> Add<T> for $wire where T: Into<$constant_type> {
            type Output = Self;

            fn add(self, c: T) -> Self {
                let e = self.composer();
                e.add_const(self, c.into())
            }
        }

        impl Sub for $wire {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                self.composer().sub(self, other)
            }
        }

        impl<T> Sub<T> for $wire where T: Into<$constant_type> {
            type Output = Self;

            fn sub(self, c: T) -> Self {
                let e = self.composer();
                e.sub_const(self, c.into())
            }
        }

        impl Neg for $wire {
            type Output = Self;

            fn neg(self) -> Self {
                let zero = self.composer().new_constant_wire(0.into());
                zero - self
            }
        }

        impl Mul for $wire {
            type Output = Self;

            fn mul(self, other: Self) -> Self {
                self.composer().mul(self, other)
            }
        }

        impl<T> Mul<T> for $wire where T: Into<$constant_type> {
            type Output = Self;

            fn mul(self, c: T) -> Self {
                let e = self.composer();
                e.mul_const(self, c.into())
            }
        }

        impl PartialEq for $wire {
            fn eq(&self, other: &Self) -> bool {
                self.composer().assert_eq(*self, *other);
                true
            }

            fn ne(&self, other: &Self) -> bool {
                self.composer().assert_ne(*self, *other);
                true
            }
        }

        impl<T: Into<$constant_type> + Clone> PartialEq<T> for $wire {
            fn eq(&self, other: &T) -> bool {
                self.composer().assert_eq_const(*self, (*other).clone().into());
                true
            }

            fn ne(&self, other: &T) -> bool {
                self.composer().assert_ne_const(*self, (*other).clone().into());
                true
            }
        }

        impl AlgWire for $wire {
            fn inv_or_panic(self) {
                self.composer().inv_or_panic(self);
            }

            fn inv_or_any(self) {
                self.composer().inv_or_any(self);
            }
        }
    };
}

/// Trait for a wire holding a boolean value
pub trait BoolWire:
    BitAnd<Output = Self> +
    BitOr<Output = Self> +
    BitXor<Output = Self> +
    Not<Output = Self> + Sized + Copy
{
    type AlgWire;

    fn to_alg(&self) -> Self::AlgWire;
    fn then_or_else(&self, then: Self::AlgWire, els: Self::AlgWire) -> Self::AlgWire;
}

#[derive(Debug, Copy, Clone)]
/// Default implementation of a boolean wire, generic over any AlgWire
pub struct Boolean<T: AlgWire>(pub T);

impl<T: AlgWire> BitAnd<Boolean<T>> for Boolean<T> {
    type Output = Boolean<T>;
    fn bitand(self, rhs: Boolean<T>) -> Boolean<T> {
        Boolean(self.0 * rhs.0)
    }
}

impl<T: AlgWire> BitOr<Boolean<T>> for Boolean<T> {
    type Output = Boolean<T>;
    fn bitor(self, rhs: Boolean<T>) -> Boolean<T> {
        Boolean(self.0 + rhs.0 - self.0 * rhs.0)
    }
}

impl<T: AlgWire> BitXor<Boolean<T>> for Boolean<T> {
    type Output = Boolean<T>;
    fn bitxor(self, rhs: Boolean<T>) -> Boolean<T> {
        Boolean(self.0 + rhs.0 * (self.0 * -2 + 1))
    }
}

impl<T: AlgWire> Not for Boolean<T> {
    type Output = Boolean<T>;
    fn not(self) -> Boolean<T> {
        Boolean(-self.0 + 1)
    }
}

impl<T: AlgWire> BoolWire for Boolean<T> {
    type AlgWire = T;

    fn to_alg(&self) -> T {
        self.0
    }

    fn then_or_else(&self, then: T, els: T) -> T {
        self.0 * then + (-self.0 + 1) * els
    }
}
