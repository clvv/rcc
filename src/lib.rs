use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;

pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;

// statements:
//   - signal / signal_input / signal_output
//   - invocation
//   - for loop

// environment:
//   - allocated cells
//   - variable to cell mapping
//   - comptime constant values

// compile(component, env) -> compute closure for component
// compute closure: read cells, write cells


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
    pub index: usize,
    context_height: usize,
    global_index: usize,
    input: bool,
}

impl Wire {
    fn new(index: usize, context_height: usize, global_index: usize, input: bool) -> Wire {
        Wire { index, context_height, global_index, input }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for Wire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // let id = format_ident!("wire_{}", self.index);
        // tokens.extend(quote! { (*#id) });
        let n = format_ident!("v_{}_{}", self.context_height, self.index);
        // tokens.extend(quote! { (*(wires.get_unchecked(#n) as *const F as *mut F)) });
        tokens.extend(quote! { (*wire(#n)) });
    }
}

/// Keeps track of environment of a single circuit component
#[derive(Default, Debug, Clone)]
struct ComponentContext {
    name: String,
    code: TokenStream,
    var_start: usize,
    allocated: usize,
    input_wires: Vec<Wire>,
    gen: bool,
}

impl ComponentContext {
    fn new(name: String, gen: bool, start: usize) -> Self {
        ComponentContext {
            name: name,
            code: TokenStream::default(),
            allocated: 0,
            input_wires: vec![],
            gen: gen,
            var_start: start
        }
    }
}

/// Environment keeping track of circuit states
#[derive(Default)]
pub struct BaseComposer {
    allocated: usize,
    circuit_body: TokenStream,
    func_defs: HashMap<String, TokenStream>,
    contextStack: Vec<ComponentContext>,
    compiledContexts: HashMap<String, ComponentContext>
}

impl Composer for BaseComposer {
    type Wire = Wire;
}

impl BaseComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.contextStack.push(ComponentContext::new("main".into(), false, 0));
        s
    }

    pub fn new_input(&mut self, w: Wire) -> Wire {
        self.allocated += 1;
        let height = self.contextStack.len();
        let context = self.contextStack.last_mut().unwrap();
        let n = context.allocated;
        context.allocated += 1;
        let w = Wire::new(n, height, w.global_index, true);
        context.input_wires.push(w);
        w
    }

    pub fn new_wire(&mut self) -> Wire {
        let m = self.allocated;
        self.allocated += 1;
        let height = self.contextStack.len();
        let context = self.contextStack.last_mut().unwrap();
        let n = context.allocated;
        context.allocated += 1;
        let w = Wire::new(n, height, m, false);
        w
        // } else {
        //     Wire::new(0, self.contextStack.len(), m)
    }

    // pub fn new_wire(&mut self) -> Wire {
    //     let n = self.allocated;
    //     self.allocated += 1;
    //     Wire::new(n)
    // }

    // pub fn new_wires(&mut self, num: usize) -> Vec<Wire> {
    //     let n = self.allocated;
    //     self.allocated += num;
    //     (n..(n + num)).map(Wire::new).collect()
    // }

    pub fn runtime(&mut self, code: TokenStream) {
        if self.contextStack.last().unwrap().gen {
            self.contextStack.last_mut().unwrap().code.extend(code.clone());
        }
        // self.circuit_body.extend(code);
    }

    pub fn register_func(&mut self, name: String, code: TokenStream) -> bool {
        if !self.func_defs.contains_key(&name) {
            self.func_defs.insert(name, code);
            true
        } else {
            false
        }
    }

    pub fn enter_context(&mut self, name: String) {
        let gen = !self.compiledContexts.contains_key(&name);
        let var_start = self.contextStack.last().unwrap().allocated + 1;
        self.contextStack.push(ComponentContext::new(name, gen, var_start));
    }

    pub fn exit_context(&mut self) {
        let height = self.contextStack.len();
        let context = self.contextStack.pop().unwrap();
        let context_name = context.name.clone();
        let name = format_ident!("{}", context.name);
        if context.gen {
            let binds = (0..context.allocated).map( |i| {
                format_ident!("v_{}_{}", height, i)
            });
            let code = context.code.clone();
            let ts = quote! {
                let #name = | #( #binds ) ,* | {
                    #code
                }
            };
            println!("{}", ts);
            self.compiledContexts.insert(context.name.clone(), context.clone());
        }
        // let new_wires_decl = context.new_wires.iter().filter(|w| !w.input).map(|w| {
        //     let gl = w.global_index;
        //     let id = format_ident!("v_{}_{}", w.context_height, w.index);
        //     quote! {
        //         let #id = #gl;
        //     }
        // });
        let input_wires = context.input_wires.iter().map(|w| {
            format_ident!("v_{}_{}", w.context_height, w.index)
        });
        self.runtime(quote! {
            // #( #new_wires_decl ) *
            #name( #( #input_wires ) ,* );
        });
    }

    pub fn add(&mut self, a: Wire, b: Wire) -> Wire {
        self.register_func("add".into(),
            quote! {
                let add_to = |i, j, k| {
                    (*wire(k)) = (*wire(i)) * (*wire(j))
                };
            }
        );

        let c = self.new_wire();
        let i = a.index;
        let j = b.index;
        let k = c.index;
        self.runtime(quote! {
            // #c = #a + #b;
            add_to(#i, #j, #k);
        });
        // TODO: constraints need to be generated here
        c
    }

    pub fn add_const(&mut self, a: Wire, b: Fp) -> Wire {
        self.enter_context("add_const".into());

        let a = self.new_input(a);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });

        self.exit_context();

        // TODO: constraints need to be generated here
        c
    }

    pub fn sub(&mut self, a: Wire, b: Wire) -> Wire {
        self.enter_context("sub".into());

        let a = self.new_input(a);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        self.exit_context();
        // TODO: constraints need to be generated here
        c
    }

    pub fn sub_const(&mut self, a: Wire, b: Fp) -> Wire {
        self.enter_context("sub_const".into());

        let a = self.new_input(a);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        self.exit_context();

        // TODO: constraints need to be generated here
        c
    }

    pub fn mul(&mut self, a: Wire, b: Wire) -> Wire {
        self.register_func("mul".into(), quote! {
            let mul_to = |i, j, k| {
                (*wire(k)) = (*wire(i)) * (*wire(j))
            };
        });
        let c = self.new_wire();
        let i = a.index;
        let j = b.index;
        let k = c.index;
        self.runtime(quote! {
            // #c = #a * #b;
            mul_to(#i, #j, #k);
        });
        // TODO: constraints need to be generated here
        c
    }

    pub fn sum(&mut self, wires: Vec<Wire>) -> Wire {
        let mut running_sum = vec![*wires.get(0).unwrap()];
        (1..wires.len()).for_each(|i| {
            running_sum.push(self.add(*running_sum.last().unwrap(), *wires.get(i).unwrap()));
        });
        *running_sum.last().unwrap()
    }

    pub fn compose_read(&mut self, wire: Wire, index: usize) {
        self.runtime(
            quote! {
                #wire = F::from(args.get(#index).unwrap().parse::<i32>().unwrap());
            }
        )
    }

    pub fn compose_log(&mut self, wire: Wire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    pub fn compose_final_circuit(&mut self) -> TokenStream {
        let n = self.allocated;
        let body = self.circuit_body.clone();

        let func_defs = self.func_defs.iter().map(|(_, x)| x);

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
                let wires: Vec<F> = vec![F::default(); #n];

                let wire = |i: usize| {
                    unsafe {
                        &mut *(wires.get_unchecked(i) as *const F as *mut F)
                    }
                };

                #( #func_defs )*

                #body

                // #intent
            }
        }
    }
}
