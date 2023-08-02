#![allow(unused_must_use)]

use num_bigint::BigUint;
use polyexen::expr::{Column, ColumnKind, ColumnQuery, Expr, PlonkVar};
use polyexen::plaf::{
    ColumnFixed, ColumnWitness, ColumnPublic, Columns, Info, Plaf, Poly, CopyC
};
use polyexen::plaf::PlafDisplayBaseTOML;


use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use indexmap::IndexMap;
pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;
use rcc::{Wire, runtime_composer::RuntimeComposer, arithmetic_logic::{AlgWire, Boolean}};

pub use rcc::Composer;
pub use rcc_macro::new_context_of;
pub type RuntimeWire = <RuntimeComposer as Composer>::Wire;

use std::ops::{Add, Sub, Mul, Neg};

fn u(n: u32) -> Option<BigUint> {
    Some(BigUint::from(n))
}

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
    constants: IndexMap<String, usize>,
    selectors: Vec<usize>,
    wires: Vec<H2Wire>,
    num_public: usize,
    public: Vec<H2Wire>,
    columns: Columns,
    copys: Vec<CopyC>
}

#[derive(Clone, Copy)]
pub struct H2Wire {
    id: usize,
    runtime_wire: RuntimeWire,
    composer_ptr: *mut H2Composer
}

impl Add for H2Wire {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.composer().add(self, other)
    }
}

impl<T> Add<T> for H2Wire where T: Into<F> {
    type Output = Self;

    fn add(self, c: T) -> Self {
        let e = self.composer();
        e.add_const(self, c.into())
    }
}

impl Sub for H2Wire {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.composer().sub(self, other)
    }
}

impl<T> Sub<T> for H2Wire where T: Into<F> {
    type Output = Self;

    fn sub(self, c: T) -> Self {
        let e = self.composer();
        e.sub_const(self, c.into())
    }
}

impl Neg for H2Wire {
    type Output = Self;

    fn neg(self) -> Self {
        let e = self.composer();
        let zero = e.new_constant_wire(0.into());
        zero - self
    }
}

impl Mul for H2Wire {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        self.composer().mul(self, other)
    }
}

impl<T> Mul<T> for H2Wire where T: Into<F> {
    type Output = Self;

    fn mul(self, c: T) -> Self {
        let e = self.composer();
        e.mul_const(self, c.into())
    }
}

impl PartialEq for H2Wire {
    fn eq(&self, other: &Self) -> bool {
        self.composer().assert_eq(*self, *other);
        true
    }

    fn ne(&self, other: &Self) -> bool {
        self.composer().assert_ne(*self, *other);
        true
    }
}

impl<T: Into<F> + Clone> PartialEq<T> for H2Wire {
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

impl Wire for H2Wire {
    type Composer = H2Composer;

    fn composer(&self) -> &mut H2Composer {
        unsafe {
            &mut *self.composer_ptr as &mut H2Composer
        }
    }
}

impl AlgWire for H2Wire {
    type Bool = Boolean<H2Wire>;

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
    fn fill_selector(&mut self) {
        let n = self.wires.len() - self.selectors.len();
        if n > 0 {
            self.selectors.extend((0..n).map(|_| 0))
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

    #[new_context_of(self)]
    /// Add gadget
    pub fn add(&mut self, a: H2Wire, b: H2Wire) -> H2Wire {
        self.fill_selector();

        // Allocate new wires. Order of allocation matters here.
        let ap = self.new_wire_from(a);
        let bp = self.new_wire_from(b);
        let one = self.new_constant_wire(1.into());

        // Generate a new runtime wire and generate runtime code to compute it from a and b
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #ap + #bp;
        });

        self.selectors.extend([1, 0, 0, 0].iter());

        c
    }

    #[new_context_of(self)]
    /// Add const gadget
    pub fn add_const(&mut self, a: H2Wire, b: F) -> H2Wire {
        self.fill_selector();

        let ap = self.new_wire_from(a);
        let bp = self.new_constant_wire(b);
        let _one = self.new_constant_wire(1.into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #ap + #bp;
        });

        self.selectors.extend([1, 0, 0, 0].iter());

        c
    }

    #[new_context_of(self)]
    /// Sub gadget
    pub fn sub(&mut self, a: H2Wire, b: H2Wire) -> H2Wire {
        self.fill_selector();

        // Allocate new wires. Order of allocation matters here.
        let ap = self.new_wire_from(a);
        let bp = self.new_wire_from(b);
        let _minus_one = self.new_constant_wire((-1).into());

        // Generate a new runtime wire and generate runtime code to compute it from a and b
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #ap - #bp;
        });

        self.selectors.extend([1, 0, 0, 0].iter());

        c
    }

    #[new_context_of(self)]
    /// Sub_const gadget
    pub fn sub_const(&mut self, a: H2Wire, b: F) -> H2Wire {
        self.fill_selector();

        let ap = self.new_wire_from(a);
        let bp = self.new_constant_wire(b);
        let _minus_one = self.new_constant_wire((-1).into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #ap - #bp;
        });

        self.selectors.extend([1, 0, 0, 0].iter());

        c
    }

    #[new_context_of(self)]
    /// Mul gadget
    pub fn mul(&mut self, a: H2Wire, b: H2Wire) -> H2Wire {
        self.fill_selector();

        let _zero = self.new_constant_wire(0.into());
        let ap = self.new_wire_from(a);
        let bp = self.new_wire_from(b);

        // Generate a new runtime wire and generate runtime code to compute it from a and b
        let c = self.new_wire();
        self.runtime(quote! {
            #c = #ap * #bp;
        });

        self.selectors.extend([1, 0, 0, 0].iter());

        c
    }

    #[new_context_of(self)]
    /// Mul const gadget
    pub fn mul_const(&mut self, a: H2Wire, b: F) -> H2Wire {
        self.fill_selector();

        let _zero = self.new_constant_wire(0.into());
        let ap = self.new_wire_from(a);
        let bp = self.new_constant_wire(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #ap * #bp;
        });

        self.selectors.extend([1, 0, 0, 0].iter());

        c
    }

    #[new_context_of(self)]
    /// Inv gadget
    /// Throws runtime error if `a` is `0`
    pub fn inv(&mut self, a: H2Wire) -> H2Wire {
        self.fill_selector();

        let _zero = self.new_constant_wire(0.into());
        let ap = self.new_wire_from(a);
        let b = self.new_wire();
        self.runtime(quote! {
            #b = #ap.inverse().unwrap();
        });

        let _one = self.new_constant_wire(1.into());

        self.selectors.extend([1, 0, 0, 0].iter());

        b
    }

    #[new_context_of(self)]
    /// Inv_zero gadget
    /// if `a` is 0 then returns `0`
    pub fn inv_zero(&mut self, a: H2Wire) -> H2Wire {
        todo!()
    }

    #[new_context_of(self)]
    /// Sum gadget
    pub fn sum(&mut self, wires: Vec<H2Wire>) -> H2Wire {
        let mut running_sum = vec![*wires.get(0).unwrap()];
        (1..wires.len()).for_each(|i| {
            running_sum.push(self.add(*running_sum.last().unwrap(), *wires.get(i).unwrap()));
        });
        *running_sum.last().unwrap()
    }

    pub fn assert_eq(&mut self, a: H2Wire, b: H2Wire) {
        self.fill_selector();

        let _ap = self.new_wire_from(a);
        let _zero = self.new_constant_wire(0.into());
        let _zero = self.new_constant_wire(0.into());
        let _bp = self.new_wire_from(b);

        self.selectors.extend([1, 0, 0, 0].iter());
    }

    pub fn assert_ne(&mut self, a: H2Wire, b: H2Wire) {
        (a - b).assert_not_zero();
    }

    /// Compose runtime code that logs the value of a wire
    pub fn log(&mut self, wire: H2Wire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    pub fn print_plaf_toml(&self) {
        println!("{}", PlafDisplayBaseTOML(&self.gen_plaf()));
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

        let selectors = self.selectors.iter().map(|&u| Some(BigUint::from(u))).collect();
        let constants = self.constants.iter().map(|(_, &u)| Some(BigUint::from(u))).collect();

        println!("selectors: {:?}", selectors);
        println!("constants: {:?}", constants);

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
