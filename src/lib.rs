use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;

mod circuit;
pub use circuit::*;

/// Comptime representation of a (constant) field element
#[derive(Default, Clone, Debug)]
pub struct Fp {
    value: F
}

impl From<u32> for Fp {
    fn from (value: u32) -> Fp {
        Fp { value: F::from(value) }
    }
}

/// A compile-time field element is translated into runtime code via this trait
impl ToTokens for Fp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let val = format!("{}", self.value.into_bigint());
        tokens.extend(quote! { F::from(BigInt!(#val)) });
    }
}

/// Composer trait
pub trait Composer {
    type Wire;
}

/// A unit of data in a circuit
#[derive(Default, Debug, Copy, Clone)]
pub struct Wire {
    index: usize,
}

impl Wire {
    fn new(index: usize) -> Wire {
        Wire { index }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for Wire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // let id = format_ident!("wire_{}", self.index);
        // tokens.extend(quote! { (*#id) });
        let n = self.index;
        // tokens.extend(quote! { (*(wires.get_unchecked(#n) as *const F as *mut F)) });
        tokens.extend(quote! { (*wire(#n)) });
    }
}

/// Keeps track of environment of a single circuit component
#[derive(Default, Debug)]
struct ComponentContext {
    closure: TokenStream,
    /// Allocated wires so far
    allocated: usize,
}

/// Environment keeping track of circuit states
#[derive(Default)]
pub struct BaseComposer {
    allocated: usize,
    circuit_body: TokenStream,
    func_defs: Vec<TokenStream>,
    componentContexts: Vec<ComponentContext>
}

impl Composer for BaseComposer {
    type Wire = Wire;
}

impl BaseComposer {
    fn new() -> Self {
        Self::default()
    }

    fn new_wire(&mut self) -> Wire {
        let n = componentContexts.last_mut().unwrap().allocated;
        componentContexts.last_mut().unwrap().allocated += 1;
        // let n = self.allocated;
        // self.allocated += 1;
        let w = Wire::new(n);
        // let id = format_ident!("wire_{}", n);
        // self.circuit_body.extend(quote! {
        //     let #id = &mut *(wires.get_unchecked(#n) as *const F as *mut F);
        // });
        w
    }

    fn new_wires(&mut self, num: usize) -> Vec<Wire> {
        let n = self.allocated;
        self.allocated += num;
        (n..(n + num))
            .map(|n| {
                // let id = format_ident!("wire_{}", n);
                // self.circuit_body.extend(quote! {
                //     let #id = &mut *(wires.get_unchecked(#n) as *const F as *mut F);
                // });
                Wire::new(n)
            })
            .collect()
    }

    fn runtime(&mut self, code: TokenStream) {
        self.component_closures.last_mut().unwrap().extend(code);
    }

    fn enter_context(&mut self, name: &str) {
        self.component_closures.push(TokenStream::default());
    }

    fn exit_context(&mut self) {
    }

    fn add(&mut self, a: Wire, b: Wire) -> Wire {
        self.enter_context("add");
        let a = self.process_input(a);
        let b = self.process_input(b);

        let c = self.new_wire();
        self.circuit_body.extend(quote! {
            #c = #a + #b;
        });
        self.exit_context();
        // let c = self.new_wire();
        // let i = a.index;
        // let j = b.index;
        // let k = c.index;
        // self.circuit_body.extend(quote! {
        //     // #c = #a + #b;
        //     add_to(#i, #j, #k);
        // });
        // // TODO: constraints need to be generated here
        // c
    }

    fn add_const(&mut self, a: Wire, b: Fp) -> Wire {
        let c = self.new_wire();
        self.circuit_body.extend(quote! {
            #c = #a + #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    fn sub(&mut self, a: Wire, b: Wire) -> Wire {
        let c = self.new_wire();
        self.circuit_body.extend(quote! {
            #c = #a - #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    fn sub_const(&mut self, a: Wire, b: Fp) -> Wire {
        let c = self.new_wire();
        self.circuit_body.extend(quote! {
            #c = #a - #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    fn mul(&mut self, a: Wire, b: Wire) -> Wire {
        let c = self.new_wire();
        let i = a.index;
        let j = b.index;
        let k = c.index;
        self.circuit_body.extend(quote! {
            // #c = #a * #b;
            mul_to(#i, #j, #k);
        });
        // TODO: constraints need to be generated here
        c
    }

    fn sum(&mut self, wires: Vec<Wire>) -> Wire {
        let mut running_sum = vec![*wires.get(0).unwrap()];
        (1..wires.len()).for_each(|i| {
            running_sum.push(self.add(*running_sum.last().unwrap(), *wires.get(i).unwrap()));
        });
        *running_sum.last().unwrap()
    }

    fn compose_read(&mut self, wire: Wire, index: usize) {
        self.circuit_body.extend(
            quote! {
                #wire = F::from(args.get(#index).unwrap().parse::<i32>().unwrap());
            }
        )
    }

    fn compose_log(&mut self, wire: Wire) {
        self.circuit_body.extend(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    fn compose_wire_allocation(&mut self) -> TokenStream {
        let n = self.allocated;
        quote! {
            let wires: Vec<F> = vec![F::default(); #n];
        }
    }

    pub fn compose_final_circuit(&mut self) -> TokenStream {
        let allocate_wires = self.compose_wire_allocation();
        let body = self.circuit_body.clone();

        let func_defs = self.func_defs.iter();

        // let intent = quote! {
        //     wires.iter().enumerate().for_each(|(i, w)| {
        //         println!("{}: {}", i, w.into_bigint())
        //     })
        // };

        quote! {
            use rcs::{F, BigInt, PrimeField};
            use std::env;

            fn main() {
                let args: Vec<String> = env::args().collect();

                #allocate_wires

                let wire = |i: usize| {
                    unsafe {
                        &mut *(wires.get_unchecked(i) as *const F as *mut F)
                    }
                };

                let mul_to = |i, j, k| {
                    (*wire(k)) = (*wire(i)) * (*wire(j))
                };

                let add_to = |i, j, k| {
                    (*wire(k)) = (*wire(i)) + (*wire(j))
                };

                #( #func_defs )*

                #body

                // #intent
            }
        }
    }
}
