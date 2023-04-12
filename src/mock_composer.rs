use proc_macro2::TokenStream;
use quote::quote;
use indexmap::IndexMap;
use crate::runtime_composer::RuntimeComposer;

pub use crate::Composer;
pub use rcc_macro::new_context_of;
pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;
pub type Wire = <RuntimeComposer as Composer>::Wire;

#[derive(Default)]
/// Mock circuit composer that implements basic add, mul, and inverse functionalities
pub struct MockComposer {
    runtime_composer: RuntimeComposer,
    constants: IndexMap<String, Wire>,
}

/// This implements numerous default functions
impl Composer for MockComposer {
    type Wire = Wire;
    type BaseComposer = RuntimeComposer;
}

impl MockComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.runtime_composer = RuntimeComposer::new();
        s
    }

    /// Allocated a new wire and return it
    pub fn new_wire(&mut self) -> Wire {
        // TODO: add wire to constraint system here
        self.runtime_composer.new_wire()
    }

    pub fn new_wires(&mut self, num: usize) -> Vec<Wire> {
        (0..num).map(|_| self.new_wire()).collect()
    }

    /// Allocated a constant wire
    pub fn new_constant_wire(&mut self, v: F) -> Wire {
        let key = format!("{}", v.into_bigint());
        if self.constants.contains_key(&key) {
            *self.constants.get(&key).unwrap()
        } else {
            let w = self.runtime_composer.new_wire();
            self.constants.insert(key, w);
            w
        }
    }

    #[new_context_of(self)]
    /// Mock add gadget
    pub fn add(&mut self, a: Wire, b: Wire) -> Wire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock add const gadget
    pub fn add_const(&mut self, a: Wire, b: F) -> Wire {
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
    pub fn sub(&mut self, a: Wire, b: Wire) -> Wire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[new_context_of(self)]
    /// Mock sub_const gadget
    pub fn sub_const(&mut self, a: Wire, b: F) -> Wire {
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
    pub fn mul(&mut self, a: Wire, b: Wire) -> Wire {
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
    pub fn inv(&mut self, a: Wire) -> Wire {
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
    pub fn inv_zero(&mut self, a: Wire) -> Wire {
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
    pub fn sum(&mut self, wires: Vec<Wire>) -> Wire {
        let mut running_sum = vec![*wires.get(0).unwrap()];
        (1..wires.len()).for_each(|i| {
            running_sum.push(self.add(*running_sum.last().unwrap(), *wires.get(i).unwrap()));
        });
        *running_sum.last().unwrap()
    }

    /// Compose runtime code that read an commandline argument into a wire
    pub fn arg_read(&mut self, wire: Wire, index: usize) {
        self.runtime(
            quote! {
                #wire = F::from(args.get(#index).unwrap().parse::<i32>().unwrap());
            }
        )
    }

    /// Compose runtime code that logs the value of a wire
    pub fn log(&mut self, wire: Wire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    /// Returns a TokenStream encoding a closure that computes all the witnesses
    pub fn compose_rust_witness_gen(&mut self) -> TokenStream {
        let prelude = quote! {
                use ark_ff::{BigInt, PrimeField};
                use ark_bn254::Fr as F;
                // runtime composer expects WireVal to be defined
                type WireVal = F;

                use std::env;
                let args: Vec<String> = env::args().collect();
        };

        let (constant_values, constant_indices): (Vec<_>, Vec<_>) = self.constants.iter().map(|(v, w)| {
            (v, w.global_index)
        }).unzip();

        let constant_decl = quote! {
            #( (*wire(#constant_indices)) = F::from(BigInt!(#constant_values)) ; ) *
        };

        self.runtime_composer.compose_rust_witness_gen(prelude, constant_decl)
    }
}
