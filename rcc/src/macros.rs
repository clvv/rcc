#[macro_export]
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
                let e = self.composer();
                let zero = e.new_constant_wire(0.into());
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

