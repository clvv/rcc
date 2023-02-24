use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
// use std::collections::HashMap;
use fixedstr::fstr;
use indexmap::IndexMap;

type Str = fstr<16>;

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
#[derive(Debug, Copy, Clone)]
pub struct Wire {
    context: Str,
    global_index: usize,
    input: bool,
    ptr_composer: *const BaseComposer
}

impl Wire {
    fn new(context: String, global_index: usize, input: bool, ptr_composer: *const BaseComposer) -> Wire {
        let context = Str::from(&context);
        Wire { context, global_index, input, ptr_composer }
    }

    fn format_against_latest_context(&self) -> TokenStream {
        unsafe {
            let e = &*self.ptr_composer as &BaseComposer;
            let last_context = e.contextStack.last().unwrap();
            let var = last_context.format_wire(*self);
            quote! {
                (*wire(#var))
            }
        }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for Wire {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.format_against_latest_context())
    }
}

/// Keeps track of environment of a single circuit component
#[derive(Default, Debug, Clone)]
struct ComponentContext {
    name: String,
    code: TokenStream,
    var_start: usize,
    global_start: usize,
    allocated: usize,
    input_wires: Vec<Wire>,
    gen: bool,
}

impl ComponentContext {
    fn new(name: String, gen: bool, var_start: usize, global_start: usize) -> Self {
        ComponentContext {
            name,
            code: TokenStream::default(),
            allocated: 0,
            input_wires: vec![],
            gen,
            var_start,
            global_start,
        }
    }

    fn input_index(&self, w: Wire) -> Option<usize> {
        self.input_wires.iter().enumerate().filter_map(|(i, a)| {
            if a.global_index == w.global_index {
                Some(i)
            } else { None }
        }).next()
    }

    fn format_wire(&self, w: Wire) -> TokenStream {
        if let Some(index) = self.input_index(w) {
            let id = format_ident!("in_{}_{}", self.name, index);
            quote! { #id }
        } else {
            let wires_var = format_ident!("wires_{}", self.name);
            let id = w.global_index - self.global_start;
            quote! {
                #wires_var[#id]
            }
        }
        // if self.global_start > w.global_index {
        //     // must be an input wire
        //     let index = self.input_index(w);
        // } else {
        // }
    }
}

/// Environment keeping track of circuit states
#[derive(Default)]
pub struct BaseComposer {
    allocated: usize,
    func_defs: IndexMap<String, TokenStream>,
    contextStack: Vec<ComponentContext>,
    compiledContexts: IndexMap<String, ComponentContext>
}

impl Composer for BaseComposer {
    type Wire = Wire;
}

impl BaseComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.contextStack.push(ComponentContext::new("".into(), true, 0, 0));
        s
    }

    pub fn new_input(&mut self, w: Wire) -> Wire {
        // let ptr = self as *const BaseComposer;
        // let w = Wire::new(self.get_latest_context_name(), w.global_index, true, ptr);
        self.contextStack.last_mut().unwrap().input_wires.push(w);
        w
    }

    pub fn new_wire(&mut self) -> Wire {
        let m = self.allocated;
        self.allocated += 1;
        let n = self.contextStack.last().unwrap().allocated;
        for context in self.contextStack.iter_mut() {
            context.allocated += 1
        }
        let context = self.contextStack.last().unwrap();
        let w = Wire::new(context.name.clone(), m, false, self as *const BaseComposer);
        w
    }

    // pub fn new_wires(&mut self, num: usize) -> Vec<Wire> {
    //     let n = self.allocated;
    //     self.allocated += num;
    //     (n..(n + num)).map(Wire::new).collect()
    // }

    pub fn runtime(&mut self, code: TokenStream) {
        if self.contextStack.last().unwrap().gen {
            self.contextStack.last_mut().unwrap().code.extend(code.clone());
        }
    }

    pub fn get_latest_context_name(&self) -> String {
        self.contextStack.last().unwrap().name.clone()
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
        let var_start = self.contextStack.last().unwrap().allocated;
        self.contextStack.push(ComponentContext::new(name, gen, var_start, self.allocated));
    }

    pub fn exit_context(&mut self) {
        let mut context = self.contextStack.pop().unwrap();
        let name = format_ident!("{}", context.name);
        let wires_var = format_ident!("wires_{}", context.name);
        if context.gen {
            let binds = context.input_wires.iter().map(|w| {
                context.format_wire(*w)
            });
            let code = context.code.clone();
            let def = quote! {
                let #name = | #wires_var: &[usize], #( #binds ) ,* | {
                    #code
                };
            };
            // println!("{}", def.clone());
            context.code = def;
            self.compiledContexts.insert(context.name.clone(), context.clone());
        }
        let prev_context = self.contextStack.last().unwrap();
        let input_wires = context.input_wires.iter().map(|w| {
            prev_context.format_wire(*w)
        });

        let wires_var = format_ident!("wires_{}", prev_context.name);
        let start = context.var_start;
        let end = context.var_start + context.allocated;

        self.runtime(quote! {
            #name( #wires_var[#start..#end].try_into().unwrap(), #( #input_wires ) ,*);
        });
    }

    pub fn add(&mut self, a: Wire, b: Wire) -> Wire {
        self.enter_context("add".into());
        let a = self.new_input(a);
        let b = self.new_input(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
            // add_to(#i, #j, #k);
        });

        self.exit_context();
        // TODO: constraints need to be generated here
        c
    }

    pub fn add_const(&mut self, a: Wire, b: Fp) -> Wire {
        // self.enter_context("add_const".into());
        // let a = self.new_input(a);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });

        // self.exit_context();

        // TODO: constraints need to be generated here
        c
    }

    pub fn sub(&mut self, a: Wire, b: Wire) -> Wire {
        // self.enter_context("sub".into());
        // let a = self.new_input(a);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // self.exit_context();
        // TODO: constraints need to be generated here
        c
    }

    pub fn sub_const(&mut self, a: Wire, b: Fp) -> Wire {
        // self.enter_context("sub_const".into());
        // let a = self.new_input(a);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // self.exit_context();

        // TODO: constraints need to be generated here
        c
    }

    pub fn mul(&mut self, a: Wire, b: Wire) -> Wire {
        self.enter_context("mul".into());
        let a = self.new_input(a);
        let b = self.new_input(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
        });

        self.exit_context();
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

        let func_defs = self.compiledContexts.iter().map(|(_, c) | {
            c.code.clone()
        });

        let context = self.contextStack.pop().unwrap();
        let main = context.code;

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
                let wires_: Vec<usize> = (0..#n).collect();

                let wire = |i: usize| {
                    unsafe {
                        &mut *(wires.get_unchecked(i) as *const F as *mut F)
                    }
                };

                #( #func_defs )*

                #main

                // #intent
            }
        }
    }
}
