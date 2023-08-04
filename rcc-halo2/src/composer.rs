#![allow(unused_variables)]
#![allow(unused_must_use)]

use num_bigint::BigUint;
use rcc::traits::AlgComposer;
use std::ops::{Add, Sub, Mul, Neg};
use polyexen::expr::{Column, ColumnKind, ColumnQuery, Expr, PlonkVar};
use polyexen::plaf::{
    ColumnFixed, ColumnWitness, ColumnPublic, Columns, Info, Plaf, Poly, CopyC
};
use polyexen::plaf::{PlafDisplayBaseTOML, PlafDisplayFixedCSV};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use indexmap::IndexMap;
pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;
use rcc::{Wire, runtime_composer::RuntimeComposer, traits::{AlgWire, Boolean}, impl_alg_op};

pub use rcc::Composer;
pub use rcc_macro::new_context_of;
pub type RuntimeWire = <RuntimeComposer as Composer>::Wire;

fn fc(index: usize) -> Column {
    Column { kind: ColumnKind::Fixed, index }
}

fn wc(index: usize) -> Column {
    Column { kind: ColumnKind::Witness, index }
}

fn pc(index: usize) -> Column {
    Column { kind: ColumnKind::Public, index }
}

#[derive(Default)]
/// We use a single vertical gate of the form
///    w(X) | s(X)
///    a    | 1
///    b    | 0
///    c    | 0
///    d    | 0
/// satisfying s(X) * (w(X) + w(X*\omega) * w(X*\omega^2) = w(X*\omega^3)) == 0
/// i.e. a + b * c = d when the selector column is turned on
pub struct H2Composer {
    runtime_composer: RuntimeComposer,
    /// Maps field element to their position in the constant column
    constants: IndexMap<String, usize>,
    /// The selector column
    selectors: Vec<usize>,
    /// The witness column, each cell is represened by a wire
    wires: Vec<H2Wire>,
    /// Number of element of the public column
    num_public: usize,
    /// Plaf columns
    columns: Columns,
    /// List of copy constraints
    copys: Vec<CopyC>
}

#[derive(Clone, Copy)]
pub struct H2Wire {
    id: usize,
    runtime_wire: RuntimeWire,
    composer_ptr: *mut H2Composer
}

impl_alg_op!(H2Wire, F);

impl Wire for H2Wire {
    type Composer = H2Composer;

    fn composer(&self) -> &mut H2Composer {
        unsafe {
            &mut *self.composer_ptr as &mut H2Composer
        }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for H2Wire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.runtime_wire.format_against_latest_context())
    }
}


/// This implements numerous default functions
impl Composer for H2Composer {
    type Wire = H2Wire;
    type BaseComposer = RuntimeComposer;

    fn base_composer(&mut self) -> Option<&mut RuntimeComposer> {
        Some(&mut self.runtime_composer)
    }

    /// Allocated a new wire and return it
    fn new_wire(&mut self) -> Self::Wire {
        let w = Self::Wire {
            id: self.wires.len(),
            runtime_wire: self.runtime_composer.new_wire(),
            composer_ptr: self as *mut H2Composer
        };
        self.wires.push(w);
        w
    }

    fn new_wire_to_column(&mut self, column: usize) -> Self::Wire {
        let w = Self::Wire {
            id: self.wires.len(),
            runtime_wire: self.runtime_composer.new_wire_to_column(column),
            composer_ptr: self as *mut H2Composer
        };
        self.wires.push(w);
        w
    }

    fn register_input(&mut self, w: Self::Wire) {
        self.base_composer().unwrap().register_input(w.runtime_wire)
    }
}

impl H2Composer {
    pub fn new() -> Self {
        let mut c = Self::default();
        c.columns = Columns {
            witness: vec![ColumnWitness::new(String::from("witness"), 0)],
            fixed: vec![ColumnFixed::new(String::from("selector")), ColumnFixed::new(String::from("constants"))],
            public: vec![ColumnPublic::new(String::from("public"))],
        };

        c.copys = vec![
            CopyC {
                columns: (wc(0), wc(0)),
                offsets: vec![]
            },
            // Copy constraints for public inputs
            CopyC {
                columns: (wc(0), pc(0)),
                offsets: vec![]
            },
            // Copy constraints for constants
            CopyC {
                columns: (wc(0), fc(1)),
                offsets: vec![]
            },
        ];

        c.runtime_composer = RuntimeComposer::new();
        c
    }

    /// Fill the selector vector until it is of the same length as the witness vector
    fn fill_selectors(&mut self) {
        let n = self.wires.len() - self.selectors.len();
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

        if a.id == self.wires.len() - 1 {
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
        self.copys[0].offsets.push((a.id, b.id));
        b
    }

    /// Add a new wire to the witness column that is constraint to `v`
    pub fn new_constant_wire(&mut self, v: F) -> H2Wire {
        let key = format!("{}", v.into_bigint());
        let constant_index = if self.constants.contains_key(&key) {
            *self.constants.get(&key).unwrap()
        } else {
            let l = self.constants.len();
            self.constants.insert(key, l);
            l
        };
        let w = self.new_wire();
        // TODO
        let v_bytes_be = v.into_bigint().to_bits_be();
        let v_code = quote!( &[ #( #v_bytes_be ) ,* ] );
        self.runtime_composer.runtime(quote!( #w = F::from(BigInt::from_bits_be(#v_code)); ));
        self.copys[2].offsets.push((w.id, constant_index));
        w
    }

    /// Copy a witness wire to the public column
    pub fn declare_public(&mut self, a: H2Wire) {
        let w = self.runtime_composer.new_wire_to_column(1);
        self.runtime_composer.runtime(quote!( #w = #a; ));
        self.copys[1].offsets.push((a.id, self.num_public));
        self.num_public += 1;
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

    pub fn gen_plaf(&self) -> Plaf {
        if self.selectors.len() != self.wires.len() {
            panic!("selector.len() = {}, wires.len() = {}", self.selectors.len(), self.wires.len());
        }

        let info = Info {
            // TODO: Remove hardcoded p
            p: BigUint::parse_bytes(b"21888242871839275222246405745257275088548364400416034343698204186575808495617", 10).unwrap(),
            num_rows: self.selectors.len(),
            challenges: vec![],
        };

        let fv = |index, rotation| Expr::Var(PlonkVar::Query(ColumnQuery { column: fc(index), rotation }));
        let wv = |index, rotation| Expr::Var(PlonkVar::Query(ColumnQuery { column: wc(index), rotation }));
        let pv = |index, rotation| Expr::Var(PlonkVar::Query(ColumnQuery { column: pc(index), rotation }));

        let exp = (wv(0, 0) + wv(0, 1) * wv(0, 2) - wv(0, 3)) * fv(0, 0);
        let poly = Poly {
            name: "main".into(),
            exp
        };

        let selectors: Vec<_> = self.selectors.iter().map(|&u| Some(BigUint::from(u))).collect();
        let mut constants: Vec<_> = self.constants.iter().map(|(_, &u)| Some(BigUint::from(u))).collect();

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

    /// Returns a TokenStream encoding a closure that computes all the witnesses
    pub fn compose_rust_witness_gen(&mut self) -> TokenStream {
        let prelude = quote! {
                use ark_ff::{BigInt, BigInteger, Field, PrimeField};
                use ark_bn254::Fr as F;
                // runtime composer expects WireVal to be defined
                type WireVal = F;
        };

        let init = quote!();

        self.runtime_composer.compose_rust_witness_gen(prelude, init)
    }
}

impl AlgComposer for H2Composer {
    type Constant = F;
    type Bool = Boolean<Self::Wire>;

    #[new_context_of(self)]
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

    #[new_context_of(self)]
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

    #[new_context_of(self)]
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

    #[new_context_of(self)]
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

    #[new_context_of(self)]
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

    #[new_context_of(self)]
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

    #[new_context_of(self)]
    fn assert_eq(&mut self, a: H2Wire, b: H2Wire) {
        let ap = self.start_gate_with(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_wire_from(b);
    }

    #[new_context_of(self)]
    fn assert_eq_const(&mut self, a: H2Wire, b: F) {
        let ap = self.start_gate_with(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_constant_wire(b);
    }

    #[new_context_of(self)]
    fn assert_ne_const(&mut self, a: H2Wire, b: F) {
        let ap = self.start_gate_with(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_constant_wire(b);
    }

    #[new_context_of(self)]
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

    #[new_context_of(self)]
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
        zero_or_one * (zero_or_one - 1) == 0;

        b
    }

    fn to_bool(&mut self, a: H2Wire) -> Self::Bool {
        let b = self.inv_or_any(a);
        Boolean(a * b)
    }

    fn check_bool(&mut self, a: H2Wire) -> Self::Bool {
        a * (a - 1) == 0;
        Boolean(a)
    }
}
