#![allow(unused_variables)]
#![allow(unused_must_use)]

pub use rcc::{
    traits::{AlgBuilder, AlgWire, Boolean, ToBits, ToBitsBuilder},
    Builder, WireLike,
};
pub use rcc_macro::{component, component_of, main_component};

use ark_bn254::Fr as F;
use ark_ff::PrimeField;
use indexmap::IndexMap;
use num_bigint::BigUint;
use polyexen::expr::{Column, ColumnKind, ColumnQuery, Expr, PlonkVar};
use polyexen::plaf::{ColumnFixed, ColumnPublic, ColumnWitness, Columns, CopyC, Info, Plaf, Poly};
use polyexen::plaf::{PlafDisplayBaseTOML, PlafDisplayFixedCSV};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rcc::runtime_composer::RuntimeComposer;
use rcc::{impl_alg_op, impl_global_builder, impl_to_bits};
use std::{
    ops::{Add, Mul, Neg, Sub},
    path::PathBuf,
};

type RuntimeWire = <RuntimeComposer as Builder>::Wire;

fn fc(index: usize) -> Column {
    Column {
        kind: ColumnKind::Fixed,
        index,
    }
}

fn wc(index: usize) -> Column {
    Column {
        kind: ColumnKind::Witness,
        index,
    }
}

fn pc(index: usize) -> Column {
    Column {
        kind: ColumnKind::Public,
        index,
    }
}

#[derive(Default, Debug)]
/// We use a single vertical gate of the form
///    w(X) | s(X)
///    a    | 1
///    b    | 0
///    c    | 0
///    d    | 0
/// satisfying s(X) * (w(X) + w(X*\omega) * w(X*\omega^2) = w(X*\omega^3)) == 0
/// i.e. a + b * c = d when the selector column is turned on
pub struct H2Builder {
    runtime_composer: RuntimeComposer,
    /// Maps field element to their position in the constant column
    constants: IndexMap<F, usize>,
    /// The selector column
    selectors: Vec<usize>,
    /// The witness column, each cell is represented by a wire
    witness: Vec<H2Wire>,
    /// The public input column, each cell is represented by a wire
    public: Vec<H2Wire>,
    /// All wires
    wires: Vec<H2Wire>,
    /// Plaf columns
    columns: Columns,
    /// List of copy constraints
    copys: Vec<CopyC>,
}

#[derive(Clone, Copy, Debug)]
pub struct H2Wire {
    column: usize,
    row: usize,
    runtime_wire: RuntimeWire,
    builder_ptr: *mut H2Builder,
}

impl_alg_op!(H2Wire, F);

impl WireLike for H2Wire {
    type Builder = H2Builder;

    fn builder(&self) -> &mut H2Builder {
        unsafe { &mut *self.builder_ptr as &mut H2Builder }
    }

    fn declare_public(self, name: &str) {
        self.builder().declare_public(self, name);
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for H2Wire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.runtime_wire.format_against_latest_context())
    }
}

impl H2Wire {
    pub fn to_ref(&self) -> TokenStream {
        let column = self.column;
        let row = self.row;
        quote! {
            WireRef { column: #column, row: #row }
        }
    }
}

/// This implements numerous default functions
impl Builder for H2Builder {
    type Wire = H2Wire;
    type BaseBuilder = RuntimeComposer;

    fn base_builder(&mut self) -> Option<&mut RuntimeComposer> {
        Some(&mut self.runtime_composer)
    }

    /// Allocated a new wire and return it
    fn new_wire(&mut self) -> Self::Wire {
        let w = self.runtime_composer.new_wire();
        self.new_wire_from_runtime_wire(w)
    }

    /// Allocate a new input wire under name `name`
    fn input_wire(&mut self, name: &str) -> Self::Wire {
        let w = self.runtime_composer.input_wire(name);
        self.new_wire_from_runtime_wire(w)
    }

    /// Allocate a new vector of input wires under name `name`
    fn input_wires(&mut self, name: &str, num: usize) -> Vec<Self::Wire> {
        let runtime_wires = self.runtime_composer.input_wires(name, num);
        runtime_wires
            .iter()
            .map(|&w| self.new_wire_from_runtime_wire(w))
            .collect()
    }

    /// Copy a witness wire to the public column
    fn declare_public(&mut self, a: H2Wire, name: &str) {
        // We create a wire representing the public input wire but it is not accessible elsewhere
        let w = H2Wire {
            // The first column is the public input column
            column: 1,
            row: self.public.len(),
            runtime_wire: self.runtime_composer.new_wire(),
            builder_ptr: self as *mut H2Builder,
        };
        self.runtime_composer.runtime(quote!( #w = #a; ));
        self.runtime_composer.declare_public(w.runtime_wire, name);
        self.copys[1].offsets.push((a.row, self.public.len()));
        self.public.push(w);
        self.wires.push(w);
    }
}

impl H2Builder {
    pub fn new() -> Self {
        let mut c = Self::default();
        c.columns = Columns {
            witness: vec![ColumnWitness::new(String::from("witness"), 0)],
            fixed: vec![
                ColumnFixed::new(String::from("selector")),
                ColumnFixed::new(String::from("constants")),
            ],
            public: vec![ColumnPublic::new(String::from("public"))],
        };

        c.copys = vec![
            CopyC {
                columns: (wc(0), wc(0)),
                offsets: vec![],
            },
            // Copy constraints for public inputs
            CopyC {
                columns: (wc(0), pc(0)),
                offsets: vec![],
            },
            // Copy constraints for constants
            CopyC {
                columns: (wc(0), fc(1)),
                offsets: vec![],
            },
        ];

        c.runtime_composer = RuntimeComposer::new();
        c
    }

    fn new_wire_from_runtime_wire(&mut self, rt_wire: RuntimeWire) -> H2Wire {
        let w = H2Wire {
            column: 0,
            row: self.witness.len(),
            runtime_wire: rt_wire,
            builder_ptr: self as *mut H2Builder,
        };
        self.witness.push(w);
        self.wires.push(w);
        w
    }

    /// Fill the selector vector until it is of the same length as the witness vector
    fn fill_selectors(&mut self) {
        let n = self.witness.len() - self.selectors.len();
        if n > 0 {
            self.selectors.extend((0..n).map(|_| 0))
        }
    }

    /// Start a gate with the wire `a`. Trace after this call will look like
    ///   w | s
    ///   a | 1
    ///   * | 0
    ///   * | 0
    ///   * | 0
    ///
    pub fn start_gate_with(&mut self, a: H2Wire) -> H2Wire {
        self.fill_selectors();

        if a.row == self.witness.len() - 1 {
            // If a is latest witness wire, we can simply start the gate at a
            *self.selectors.last_mut().unwrap() = 1;
            self.selectors.extend([0, 0, 0].iter());
            a
        } else {
            // If not, we must copy a in
            self.selectors.extend([1, 0, 0, 0].iter());
            self.new_wire_from(a)
        }
    }

    /// Add a new wire to the witness Column whose value is copied from a
    pub fn new_wire_from(&mut self, a: H2Wire) -> H2Wire {
        let b = self.new_wire();
        self.runtime_composer.runtime(quote!( #b = #a; ));
        self.copys[0].offsets.push((a.row, b.row));
        b
    }

    /// Add a new wire to the witness column that is constraint to `v`
    pub fn new_constant_wire(&mut self, v: F) -> H2Wire {
        let constant_index = if self.constants.contains_key(&v) {
            *self.constants.get(&v).unwrap()
        } else {
            let l = self.constants.len();
            self.constants.insert(v, l);
            l
        };
        let w = self.new_wire();
        let us = format!("{}", v.into_bigint());
        self.runtime_composer
            .runtime(quote!( #w = F::from(BigInt!(#us)); ));
        self.copys[2].offsets.push((w.row, constant_index));
        w
    }

    /// Compose runtime code that logs the value of a wire
    pub fn log(&mut self, wire: H2Wire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    pub fn print_plaf_toml(&self) {
        let plaf = self.gen_plaf();
        println!("Fixed column CSV:\n{}", PlafDisplayFixedCSV(&plaf));
        println!("Plaf TOML:\n{}", PlafDisplayBaseTOML(&plaf));
    }

    pub fn format_plaf(&self) -> String {
        let plaf = self.gen_plaf();
        toml::to_string(&plaf).unwrap()
    }

    pub fn gen_plaf(&self) -> Plaf {
        if self.selectors.len() != self.witness.len() {
            panic!(
                "selector.len() = {}, wires.len() = {}",
                self.selectors.len(),
                self.witness.len()
            );
        }

        let info = Info {
            // TODO: Remove hardcoded p
            p: BigUint::parse_bytes(
                b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
                10,
            )
            .unwrap(),
            num_rows: self.selectors.len(),
            challenges: vec![],
        };

        let fv = |index, rotation| {
            Expr::Var(PlonkVar::Query(ColumnQuery {
                column: fc(index),
                rotation,
            }))
        };
        let wv = |index, rotation| {
            Expr::Var(PlonkVar::Query(ColumnQuery {
                column: wc(index),
                rotation,
            }))
        };
        let pv = |index, rotation| {
            Expr::Var(PlonkVar::Query(ColumnQuery {
                column: pc(index),
                rotation,
            }))
        };

        let exp = (wv(0, 0) + wv(0, 1) * wv(0, 2) - wv(0, 3)) * fv(0, 0);
        let poly = Poly {
            name: "main".into(),
            exp,
        };

        let selectors: Vec<_> = self
            .selectors
            .iter()
            .map(|&u| Some(BigUint::from(u)))
            .collect();
        let mut constants: Vec<_> = self
            .constants
            .iter()
            .map(|(c, _)| Some((*c).into()))
            .collect();

        let n = selectors.len() - constants.len();
        if n > 0 {
            constants.extend((0..n).map(|_| None))
        }

        Plaf {
            info,
            columns: self.columns.clone(),
            polys: vec![poly],
            metadata: Default::default(),
            lookups: vec![],
            shuffles: vec![],
            copys: self.copys.clone(),
            fixed: vec![selectors, constants],
        }
    }

    /// Returns a String encoding a closure that computes all the witnesses
    pub fn compose_rust_witness_gen(&mut self) -> String {
        let wirerefs: Vec<TokenStream> = self.wires.iter().map(|w| w.to_ref()).collect();
        let num_rows_each_column = vec![self.witness.len(), self.public.len()];

        let prelude = quote! {
            use rcc_halo2::runtime::{WireVal, WireRef, F, *};

            static ID_TO_REF: &[WireRef] = &[ #( #wirerefs ) ,* ];
        };

        let init = quote! {
            // Allocate the wires
            let mut wires: Vec<Vec<WireVal>> = vec![];
            #( wires.push(vec![WireVal::default(); #num_rows_each_column]) ) ;* ;

            let wire = |id: usize| {
                let wf = ID_TO_REF[id];
                unsafe {
                    &mut *(
                        wires
                        .get_unchecked(wf.column)
                        .get_unchecked(wf.row) as *const WireVal as *mut WireVal
                    )
                }
            };
        };

        self.runtime_composer
            .compose_rust_witness_gen(prelude, init)
    }

    /// Write out circuit config to path
    pub fn write_config(&mut self, plaf_path: PathBuf, runtime_lib_path: PathBuf) {
        use crate::plaf::serialize;

        let plaf = serialize(self.gen_plaf());
        std::fs::write(plaf_path, plaf).expect("Unable to write file");

        let code = self.compose_rust_witness_gen();
        std::fs::write(runtime_lib_path, code).expect("Unable to write file");
    }

    pub fn compile_from_commandline(&mut self, source: &str) {
        use clap::Parser;
        let args = crate::utils::CompilationArgs::parse();

        let plaf_path = if let Some(p) = args.config {
            p
        } else {
            crate::utils::rs_path_to_config(source)
        };

        let runtime_lib_path = if let Some(p) = args.runtime {
            p
        } else {
            crate::utils::rs_path_to_runtime_lib(source)
        };

        self.write_config(plaf_path, runtime_lib_path);
    }

    /// Returns d such that c + a * b = d
    pub fn mul_add(&mut self, a: H2Wire, b: H2Wire, c: H2Wire) -> H2Wire {
        self.start_gate_with(c);
        self.new_wire_from(a);
        self.new_wire_from(b);
        let d = self.new_wire();

        self.runtime(quote! {
            #d = #a * #b + #c;
        });

        d
    }
}

impl_global_builder!(H2Builder, H2Wire);

impl AlgBuilder for H2Builder {
    type Constant = F;
    type Bool = Boolean<Self::Wire>;

    #[component_of(self)]
    /// Add gadget
    fn add(&mut self, a: H2Wire, b: H2Wire) -> H2Wire {
        let ap = self.start_gate_with(a);
        let bp = self.new_wire_from(b);
        let _one = self.new_constant_wire(1.into());
        let c = self.new_wire();

        self.runtime(quote! {
            #c = #ap + #bp;
        });

        c
    }

    #[component_of(self)]
    /// Add const gadget
    fn add_const(&mut self, a: H2Wire, b: F) -> H2Wire {
        let ap = self.start_gate_with(a);
        let bp = self.new_constant_wire(b);
        let _one = self.new_constant_wire(1.into());
        let c = self.new_wire();

        self.runtime(quote! {
            #c = #ap + #bp;
        });

        c
    }

    #[component_of(self)]
    /// Sub gadget
    fn sub(&mut self, a: H2Wire, b: H2Wire) -> H2Wire {
        let ap = self.start_gate_with(a);
        let bp = self.new_wire_from(b);
        let _minus_one = self.new_constant_wire((-1).into());
        let c = self.new_wire();

        self.runtime(quote! {
            #c = #ap - #bp;
        });

        c
    }

    #[component_of(self)]
    /// Sub_const gadget
    fn sub_const(&mut self, a: H2Wire, b: F) -> H2Wire {
        let ap = self.start_gate_with(a);
        let bp = self.new_constant_wire(b);
        let _minus_one = self.new_constant_wire((-1).into());
        let c = self.new_wire();

        self.runtime(quote! {
            #c = #ap - #bp;
        });

        c
    }

    #[component_of(self)]
    /// Mul gadget
    fn mul(&mut self, a: H2Wire, b: H2Wire) -> H2Wire {
        let zero = self.new_constant_wire(0.into());
        let zero = self.start_gate_with(zero);
        let ap = self.new_wire_from(a);
        let bp = self.new_wire_from(b);
        let c = self.new_wire();

        self.runtime(quote! {
            #c = #ap * #bp;
        });

        c
    }

    #[component_of(self)]
    /// Mul const gadget
    fn mul_const(&mut self, a: H2Wire, b: F) -> H2Wire {
        let zero = self.new_constant_wire(0.into());
        let zero = self.start_gate_with(zero);
        let ap = self.new_wire_from(a);
        let bp = self.new_constant_wire(b);
        let c = self.new_wire();

        self.runtime(quote! {
            #c = #ap * #bp;
        });

        c
    }

    #[component_of(self)]
    fn assert_eq(&mut self, a: H2Wire, b: H2Wire) {
        let ap = self.start_gate_with(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_wire_from(b);
    }

    #[component_of(self)]
    fn assert_eq_const(&mut self, a: H2Wire, b: F) {
        let ap = self.start_gate_with(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_constant_wire(b);
    }

    #[component_of(self)]
    fn assert_ne_const(&mut self, a: H2Wire, b: F) {
        let ap = self.start_gate_with(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_constant_wire(b);
    }

    #[component_of(self)]
    /// Inv gadget
    /// If `a` is `0` at runtime, constraint system cannot be satisfied
    fn inv_or_panic(&mut self, a: H2Wire) -> H2Wire {
        let zero = self.new_constant_wire(0.into());
        let zero = self.start_gate_with(zero);
        let ap = self.new_wire_from(a);
        let b = self.new_wire();
        let _one = self.new_constant_wire(1.into());

        self.runtime(quote! {
            #b = #ap.inverse()..unwrap_or(0.into());
        });

        b
    }

    #[component_of(self)]
    /// Inv gadget
    /// If `a` is `0` at runtime, return wire can hold arbitrary value
    fn inv_or_any(&mut self, a: H2Wire) -> H2Wire {
        let zero_or_one = self.new_wire();
        let zero_or_one = self.start_gate_with(zero_or_one);
        let ap = self.new_wire_from(a);
        let b = self.new_wire();
        let _one = self.new_constant_wire(1.into());

        self.runtime(quote! {
            if Some(inv) = #ap.inverse() {
                #b = inv;
                #zero_or_one = 0.into();
            } else {
                #b = 0.into();
                #zero_or_one = 1.into();
            }
        });

        // asssert that zero_or_one is 0 or 1
        self.assert_bool(zero_or_one);

        b
    }

    fn to_bool(&mut self, a: H2Wire) -> Self::Bool {
        let b = self.inv_or_any(a);
        Boolean(a * b)
    }

    #[component_of(self)]
    /// We assert a is bool by showing that 0 + a * a = a
    fn assert_bool(&mut self, a: H2Wire) -> Self::Bool {
        let zero = self.new_constant_wire(F::from(0));
        self.start_gate_with(zero);
        self.new_wire_from(a);
        self.new_wire_from(a);
        self.new_wire_from(a);
        Boolean(a)
    }

    #[component_of(self)]
    fn inner_product(&mut self, a: Vec<Self::Wire>, b: Vec<Self::Wire>) -> Self::Wire {
        let mut carry = self.mul(a[0], b[0]);
        self.smart_map(a.iter().skip(1).zip(b.iter().skip(1)), |e, (&a, &b)| {
            carry = e.mul_add(a, b, carry);
        });
        carry
    }
}

impl ToBitsBuilder for H2Builder {
    const NUM_BITS: usize = 254;

    #[component_of(self)]
    fn to_bits_be(&mut self, w: H2Wire, num_bits: usize) -> Vec<Self::Bool> {
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
            println!("{:?}", bits);
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

        let zero = self.new_constant_wire(0.into());

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

    fn to_bits_be_strict(&mut self, w: H2Wire) -> Vec<Self::Bool> {
        // TODO: additionally constrain that `bits` is less than `p`
        self.to_bits_be(w, Self::NUM_BITS)
    }

    fn to_bits_le(&mut self, w: H2Wire, num_bits: usize) -> Vec<Self::Bool> {
        let mut v = self.to_bits_be(w, num_bits);
        v.reverse();
        v
    }

    fn to_bits_le_strict(&mut self, w: H2Wire) -> Vec<Self::Bool> {
        let mut v = self.to_bits_be_strict(w);
        v.reverse();
        v
    }

    fn from_bits_be(&mut self, _: Vec<Self::Bool>) -> Self {
        todo!()
    }

    fn from_bits_le(&mut self, _: Vec<Self::Bool>) -> Self {
        todo!()
    }
}

impl_to_bits!(H2Builder, H2Wire);
