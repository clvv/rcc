use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use indexmap::IndexMap;
use rcc::{Wire, runtime_composer::RuntimeComposer, traits::{AlgWire, Boolean, AlgComposer}, impl_alg_op};

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

impl Wire for MockWire {
    type Composer = MockComposer;

    fn composer(&self) -> &mut MockComposer {
        unsafe {
            &mut *self.composer_ptr as &mut MockComposer
        }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for MockWire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.runtime_wire.format_against_latest_context())
    }
}

impl_alg_op!(MockWire, F);

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
                type Input = Vec<F>;
                type AllWires = Vec<F>;
        };

        let n = self.runtime_composer.wires.len();

        let (constant_values, constant_indices): (Vec<_>, Vec<_>) = self.constants.iter().map(|(v, w)| {
            (v, w.runtime_wire.global_id)
        }).unzip();

        let constant_decl = quote! {
            #( (*wire(#constant_indices)) = F::from(BigInt!(#constant_values)) ; ) *
        };

        let init = quote! {
            let wires: Vec<WireVal> = vec![WireVal::default(); #n];

            let wire = |id: usize| {
                unsafe {
                    &mut *(wires.get_unchecked(id) as *const WireVal as *mut WireVal)
                }
            };

            #constant_decl;
        };

        self.runtime_composer.compose_rust_witness_gen(prelude, init)
    }
}

impl AlgComposer for MockComposer {
    type Constant = F;
    type Bool = Boolean<Self::Wire>;

    #[new_context_of(self)]
    /// Mock add gadget
    fn add(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock add const gadget
    fn add_const(&mut self, a: MockWire, b: F) -> MockWire {
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
    fn sub(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock sub_const gadget
    fn sub_const(&mut self, a: MockWire, b: F) -> MockWire {
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
    fn mul(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock sub_const gadget
    fn mul_const(&mut self, a: MockWire, b: F) -> MockWire {
        let b = self.new_constant_wire(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock inv gadget
    fn inv_or_panic(&mut self, a: MockWire) -> MockWire {
        let b = self.new_wire();
        self.runtime(quote! {
            #b = #a.inverse().unwrap();
        });

        // TODO: constraints need to be generated here
        b
    }

    #[new_context_of(self)]
    fn inv_or_any(&mut self, a: MockWire) -> MockWire {
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

    fn assert_eq(&mut self, _a: MockWire, _b: MockWire) {
        // TODO: generate constraints
    }

    fn assert_eq_const(&mut self, _a: MockWire, _b: F) {
        // TODO: generate constraints
    }

    fn assert_ne(&mut self, a: MockWire, b: MockWire) {
        (a - b).inv_or_panic();
    }

    fn to_bool(&mut self, a: MockWire) -> Self::Bool {
        let b = self.inv_or_any(a);
        Boolean(a * b)
    }

    fn check_bool(&mut self, a: MockWire) -> Self::Bool {
        a * (a - 1) == 0;
        Boolean(a)
    }
}
