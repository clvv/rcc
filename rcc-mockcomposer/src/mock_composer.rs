use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use indexmap::IndexMap;
use rcc::{Wire, runtime_composer::RuntimeComposer, arithmetic_logic::{AlgWire, Boolean}};

pub use rcc::Composer;
pub use rcc_macro::new_context_of;
pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;
pub type RuntimeWire = <RuntimeComposer as Composer>::Wire;

use std::ops::{Add, Sub, Mul, Neg};

#[derive(Default)]
/// Mock circuit composer that implements basic add, mul, and inverse functionalities
pub struct MockComposer {
    runtime_composer: RuntimeComposer,
    constants: IndexMap<String, MockWire>,
}

#[derive(Clone, Copy)]
pub struct MockWire {
    runtime_wire: RuntimeWire,
    composer_ptr: *mut MockComposer
}

impl Add for MockWire {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.composer().add(self, other)
    }
}

impl<T> Add<T> for MockWire where T: Into<F> {
    type Output = Self;

    fn add(self, c: T) -> Self {
        let e = self.composer();
        let w = e.new_constant_wire(c.into());
        e.add(self, w)
    }
}

impl Sub for MockWire {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.composer().sub(self, other)
    }
}

impl<T> Sub<T> for MockWire where T: Into<F> {
    type Output = Self;

    fn sub(self, c: T) -> Self {
        let e = self.composer();
        let w = e.new_constant_wire(c.into());
        e.sub(self, w)
    }
}

impl Neg for MockWire {
    type Output = Self;

    fn neg(self) -> Self {
        let e = self.composer();
        let zero = e.new_constant_wire(0.into());
        zero - self
    }
}

impl Mul for MockWire {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.composer().mul(self, other)
    }
}

impl<T> Mul<T> for MockWire where T: Into<F> {
    type Output = Self;

    fn mul(self, c: T) -> Self {
        let e = self.composer();
        let w = e.new_constant_wire(c.into());
        e.mul(self, w)
    }
}

impl PartialEq for MockWire {
    fn eq(&self, other: &Self) -> bool {
        self.composer().assert_eq(*self, *other);
        true
    }

    fn ne(&self, other: &Self) -> bool {
        self.composer().assert_ne(*self, *other);
        true
    }
}

impl<T: Into<F> + Clone> PartialEq<T> for MockWire {
    fn eq(&self, other: &T) -> bool {
        let e = self.composer();
        let w = e.new_constant_wire((*other).clone().into());
        e.assert_eq(*self, w);
        true
    }

    fn ne(&self, other: &T) -> bool {
        let e = self.composer();
        let w = e.new_constant_wire((*other).clone().into());
        e.assert_ne(*self, w);
        true
    }
}

impl Wire for MockWire {
    type Composer = MockComposer;

    fn composer(&self) -> &mut MockComposer {
        unsafe {
            &mut *self.composer_ptr as &mut MockComposer
        }
    }
}

impl AlgWire for MockWire {
    type Bool = Boolean<MockWire>;

    /// Maps any non-zero field element to one and zero to zero.
    fn to_bool(self) -> Self::Bool {
        let w_inv = self.inv_or_default(1);
        w_inv.assert_not_zero();
        Boolean { wire: self * w_inv }
    }

    /// Assert that the wire is boolean
    fn check_bool(self) -> Self::Bool {
        self * (self - 1) == 0;
        Boolean { wire: self }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for MockWire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.runtime_wire.format_against_latest_context())
    }
}


/// This implements numerous default functions
impl Composer for MockComposer {
    type Wire = MockWire;
    type BaseComposer = RuntimeComposer;

    fn base_composer(&mut self) -> Option<&mut RuntimeComposer> {
        Some(&mut self.runtime_composer)
    }

    /// Allocated a new wire and return it
    fn new_wire(&mut self) -> Self::Wire {
        // TODO: add wire to constraint system here
        Self::Wire {
            runtime_wire: self.runtime_composer.new_wire(),
            composer_ptr: self as *mut MockComposer
        }
    }

    fn new_wire_to_column(&mut self, _: usize) -> Self::Wire { todo!() }

    fn register_input(&mut self, w: Self::Wire) {
        self.base_composer().unwrap().register_input(w.runtime_wire)
    }
}

impl MockComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.runtime_composer = RuntimeComposer::new();
        s
    }

    /// Allocated a constant wire
    pub fn new_constant_wire(&mut self, v: F) -> MockWire {
        let key = format!("{}", v.into_bigint());
        if self.constants.contains_key(&key) {
            *self.constants.get(&key).unwrap()
        } else {
            let w = self.new_wire();
            self.constants.insert(key, w);
            w
        }
    }

    #[new_context_of(self)]
    /// Mock add gadget
    pub fn add(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock add const gadget
    pub fn add_const(&mut self, a: MockWire, b: F) -> MockWire {
        let b = self.new_constant_wire(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock sub gadget
    pub fn sub(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock sub_const gadget
    pub fn sub_const(&mut self, a: MockWire, b: F) -> MockWire {
        let b = self.new_constant_wire(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock mul gadget
    pub fn mul(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock inv gadget
    /// Throws runtime error if `a` is `0`
    pub fn inv(&mut self, a: MockWire) -> MockWire {
        let b = self.new_wire();
        self.runtime(quote! {
            #b = #a.inverse().unwrap();
        });

        // TODO: constraints need to be generated here
        b
    }

    #[new_context_of(self)]
    /// Mock inv_zero gadget
    /// if `a` is 0 then returns `0`
    pub fn inv_zero(&mut self, a: MockWire) -> MockWire {
        let b = self.new_wire();
        self.runtime(quote! {
            if let Some(v) = #a.inverse() {
                #b = v;
            } else {
                #b = F::from(0);
            }
        });

        // TODO: constraints need to be generated here
        b
    }

    #[new_context_of(self)]
    /// Mock sum gadget
    pub fn sum(&mut self, wires: Vec<MockWire>) -> MockWire {
        let mut running_sum = vec![*wires.get(0).unwrap()];
        (1..wires.len()).for_each(|i| {
            running_sum.push(self.add(*running_sum.last().unwrap(), *wires.get(i).unwrap()));
        });
        *running_sum.last().unwrap()
    }

    pub fn assert_eq(&mut self, _a: MockWire, _b: MockWire) {
        // TODO: generate constraints
    }

    pub fn assert_ne(&mut self, a: MockWire, b: MockWire) {
        (a - b).assert_not_zero();
    }

    /// Compose runtime code that read an commandline argument into a wire
    pub fn arg_read(&mut self, wire: MockWire, index: usize) {
        self.runtime(
            quote! {
                #wire = F::from(args.get(#index).unwrap().parse::<i32>().unwrap());
            }
        )
    }

    /// Compose runtime code that logs the value of a wire
    pub fn log(&mut self, wire: MockWire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    /// Returns a TokenStream encoding a closure that computes all the witnesses
    pub fn compose_rust_witness_gen(&mut self) -> TokenStream {
        let prelude = quote! {
                use ark_ff::{BigInt, Field, PrimeField};
                use ark_bn254::Fr as F;
                // runtime composer expects WireVal to be defined
                type WireVal = F;
        };

        let (constant_values, constant_indices): (Vec<_>, Vec<_>) = self.constants.iter().map(|(v, w)| {
            (v, w.runtime_wire.to_ref())
        }).unzip();

        let constant_decl = quote! {
            #( (*wire(#constant_indices)) = F::from(BigInt!(#constant_values)) ; ) *
        };

        self.runtime_composer.compose_rust_witness_gen(prelude, constant_decl)
    }
}
