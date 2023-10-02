use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rcc::{runtime_composer::{Composer, RuntimeComposer, RuntimeWire}, traits::BoolWire};

pub use rcc::{
    impl_alg_op,
    impl_to_bits,
    impl_global_builder,
    Builder, WireLike,
    traits::{ToBits, AlgBuilder, AlgWire, Boolean, ToBitsBuilder},
};
pub use rcc_macro::{component, component_of, main_component};

use ark_bn254::Fr as F;
use ark_ff::PrimeField;

use std::ops::{Add, Mul, Neg, Sub};

#[derive(Default)]
/// Mock circuit composer that implements basic add, mul, and inverse functionalities
pub struct MockBuilder {
    runtime_composer: RuntimeComposer,
    constants: IndexMap<String, MockWire>,
}

#[derive(Clone, Copy)]
pub struct MockWire {
    runtime_wire: RuntimeWire,
    builder_ptr: *mut MockBuilder,
}

impl WireLike for MockWire {
    type Builder = MockBuilder;

    fn builder(&self) -> &mut MockBuilder {
        unsafe { &mut *self.builder_ptr as &mut MockBuilder }
    }

    fn declare_public(self, name: &str) {
        self.builder().declare_public(self, name);
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
impl Builder for MockBuilder {
    type Wire = MockWire;
    type Composer = RuntimeComposer;

    fn composer(&mut self) -> Option<&mut RuntimeComposer> {
        Some(&mut self.runtime_composer)
    }

    /// Allocated a new wire and return it
    fn new_wire(&mut self) -> Self::Wire {
        // TODO: add wire to constraint system here
        let w = self.runtime_composer.new_wire();
        self.new_wire_from_runtime_wire(w)
    }

    fn input_wire(&mut self, name: &str) -> Self::Wire {
        let w = self.runtime_composer.input_wire(name);
        self.new_wire_from_runtime_wire(w)
    }

    fn input_wires(&mut self, name: &str, n: usize) -> Vec<Self::Wire> {
        let ws = self.runtime_composer.input_wires(name, n);
        ws.iter()
            .map(|&w| self.new_wire_from_runtime_wire(w))
            .collect()
    }

    fn declare_public(&mut self, w: Self::Wire, name: &str) {
        self.runtime_composer.declare_public(w.runtime_wire, name);
    }
}

impl MockBuilder {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.runtime_composer = RuntimeComposer::new();
        s
    }

    fn new_wire_from_runtime_wire(&mut self, w: RuntimeWire) -> MockWire {
        MockWire {
            runtime_wire: w,
            builder_ptr: self as *mut MockBuilder,
        }
    }

    /// Compose runtime code that read an commandline argument into a wire
    pub fn arg_read(&mut self, wire: MockWire, index: usize) {
        self.runtime(quote! {
            #wire = F::from(args.get(#index).unwrap().parse::<i32>().unwrap());
        })
    }

    /// Compose runtime code that logs the value of a wire
    pub fn log(&mut self, wire: MockWire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    /// Returns a String encoding a closure that computes all the witnesses
    pub fn compose_rust_witness_gen(&mut self) -> String {
        let prelude = quote! {
            use rcc_mockbuilder::runtime::*;
        };

        let n = self.runtime_composer.wires.len();

        let (constant_values, constant_indices): (Vec<_>, Vec<_>) = self
            .constants
            .iter()
            .map(|(v, w)| (v, w.runtime_wire.global_id))
            .unzip();

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

        self.runtime_composer.compose_witness_gen(prelude, init)
    }

    pub fn compile_from_commandline(&mut self, source: &str) {
        let path = std::path::PathBuf::from(source);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let mut runtime_lib_path = path.clone();
        runtime_lib_path.set_file_name(format!("{name}_runtime_lib.rs"));

        // Compose the rust witness gen code
        let code = self.compose_rust_witness_gen();
        std::fs::write(runtime_lib_path, code).expect("Unable to write file");
    }
}

impl AlgBuilder for MockBuilder {
    type Constant = F;
    type Bool = Boolean<Self::Wire>;

    /// Allocated a constant wire
    fn new_constant_wire(&mut self, v: F) -> MockWire {
        let key = format!("{}", v.into_bigint());
        if self.constants.contains_key(&key) {
            *self.constants.get(&key).unwrap()
        } else {
            let w = self.new_wire();
            self.constants.insert(key, w);
            w
        }
    }

    #[component_of(self)]
    /// Mock add gadget
    fn add(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[component_of(self)]
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

    #[component_of(self)]
    /// Mock sub gadget
    fn sub(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    #[component_of(self)]
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

    #[component_of(self)]
    /// Mock mul gadget
    fn mul(&mut self, a: MockWire, b: MockWire) -> MockWire {
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    #[component_of(self)]
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

    #[component_of(self)]
    /// Mock inv gadget
    fn inv_or_panic(&mut self, a: MockWire) -> MockWire {
        let b = self.new_wire();
        self.runtime(quote! {
            #b = #a.inverse().unwrap();
        });

        // TODO: constraints need to be generated here
        b
    }

    #[component_of(self)]
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

    fn assert_bool(&mut self, a: MockWire) -> Self::Bool {
        a * (a - 1) == 0;
        Boolean(a)
    }
}

impl_global_builder!(MockBuilder, MockWire);

impl ToBitsBuilder for MockBuilder {
    const NUM_BITS: usize = 254;

    #[component_of(self)]
    fn to_bits_be(&mut self, w: MockWire, num_bits: usize) -> Vec<Self::Bool> {
        assert!(num_bits <= Self::NUM_BITS);

        let alg_bits = self.new_wires(num_bits);

        // Runtime code to compute be bits
        self.runtime(quote! {
            let u: BigUint = #w.into();
            let base2_bits = u.to_radix_be(2);
            let mut bits = vec![];
            if base2_bits.len() <= #num_bits {
                bits.extend((0..(#num_bits - base2_bits.len())).map(|_| F::from(0)));
                bits.extend(base2_bits.iter().map(|&i| F::from(i)));
            } else {
                panic!("Error: number has more bits than expected.")
            }
        });

        let bits = alg_bits
            .iter()
            .enumerate()
            .map(|(i, &b)| {
                self.runtime(quote! {
                    #b = bits[#i].into();
                });
                self.assert_bool(b)
            })
            .collect();

        let mut carry = F::from(1);
        let mut pow2 = vec![];
        (0..num_bits).for_each(|_| {
            pow2.push(self.new_constant_wire(carry));
            carry *= F::from(2);
        });
        pow2.reverse();

        w == self.inner_product(pow2, alg_bits);

        bits
    }

    fn to_bits_be_strict(&mut self, w: MockWire) -> Vec<Self::Bool> {
        // TODO: additionally constrain that `bits` is less than `p`
        self.to_bits_be(w, Self::NUM_BITS)
    }

    fn to_bits_le(&mut self, w: MockWire, num_bits: usize) -> Vec<Self::Bool> {
        let mut v = self.to_bits_be(w, num_bits);
        v.reverse();
        v
    }

    fn to_bits_le_strict(&mut self, w: MockWire) -> Vec<Self::Bool> {
        let mut v = self.to_bits_be_strict(w);
        v.reverse();
        v
    }

    fn from_bits_be(&mut self, bits: Vec<Self::Bool>) -> Self::Wire {
        let v = self.new_wire();
        let num_bits = bits.len();

        let mut carry = F::from(1);
        let mut pow2 = vec![];
        (0..num_bits).for_each(|_| {
            pow2.push(self.new_constant_wire(carry));
            carry *= F::from(2);
        });
        pow2.reverse();

        for (i, bit) in bits.iter().enumerate() {
            let alg_bit = bit.to_alg();
            let c = pow2[i];
            self.runtime(quote! {
                #v = #v + #alg_bit * #c;
            });
        }
        v
    }

    fn from_bits_le(&mut self, mut bits: Vec<Self::Bool>) -> Self::Wire {
        bits.reverse();
        self.from_bits_be(bits)
    }
}

impl_to_bits!(MockBuilder, MockWire);
