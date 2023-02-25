use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
// use std::collections::HashMap;
use indexmap::IndexMap;

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
    global_index: usize,
    ptr_composer: *mut BaseComposer
}

impl Wire {
    fn new(global_index: usize, ptr_composer: *mut BaseComposer) -> Wire {
        Wire { global_index, ptr_composer }
    }

    fn format_against_latest_context(&self) -> TokenStream {
        unsafe {
            let e = &mut *self.ptr_composer as &mut BaseComposer;
            let last_context = e.context_stack.last_mut().unwrap();
            let id = last_context.format_and_allocate_wire(*self);
            quote! { (*wire(#id)) }
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
    closure_name: String,
    code: TokenStream,
    allocated: usize,
    var_start: usize,
    global_start: usize,
    input_wires: Vec<Wire>,
}

impl ComponentContext {
    fn new(name: String, var_start: usize, global_start: usize) -> Self {
        ComponentContext {
            name,
            closure_name: String::default(),
            code: TokenStream::default(),
            allocated: 0,
            input_wires: vec![],
            var_start,
            global_start,
        }
    }

    fn input_index(&self, w: Wire) -> Option<usize> {
        self.input_wires.iter().enumerate().find_map(|(i, a)| {
            if a.global_index == w.global_index {
                Some(i)
            } else { None }
        })
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
    }

    fn format_and_allocate_wire(&mut self, w: Wire) -> TokenStream {
        if self.global_start > w.global_index {
            // must be an input wire
            if self.input_index(w) == None {
                self.input_wires.push(w);
            }
            let id = format_ident!("in_{}_{}", self.name, self.input_index(w).unwrap());
            quote! { #id }
        } else {
            let wires_var = format_ident!("wires_{}", self.name);
            let id = w.global_index - self.global_start;
            quote! {
                #wires_var[#id]
            }
        }
    }
}

pub struct ContextMarker {
    func: Box<dyn Fn() -> ()>
}

impl ContextMarker {
    fn new(func: Box<dyn Fn() -> ()>) -> Self {
        Self { func }
    }
}

impl Drop for ContextMarker {
    fn drop(&mut self) {
        (self.func)();
    }
}

/// Environment keeping track of circuit states
#[derive(Default)]
pub struct BaseComposer {
    context_stack: Vec<ComponentContext>,
    compiled_contexts: IndexMap<String, ComponentContext>
}

impl Composer for BaseComposer {
    type Wire = Wire;
}

impl BaseComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.context_stack.push(ComponentContext::new("".into(), 0, 0));
        s
    }

    pub fn new_wire(&mut self) -> Wire {
        let m = self.context_stack.first().unwrap().allocated;
        for context in self.context_stack.iter_mut() {
            context.allocated += 1
        }
        Wire::new(m, self as *mut BaseComposer)
    }

    // pub fn new_wires(&mut self, num: usize) -> Vec<Wire> {
    //     let n = self.allocated;
    //     self.allocated += num;
    //     (n..(n + num)).map(Wire::new).collect()
    // }

    pub fn runtime(&mut self, code: TokenStream) {
        // if self.contextStack.last().unwrap().gen {
        self.context_stack.last_mut().unwrap().code.extend(code.clone());
        // }
    }

    pub fn get_latest_context_name(&self) -> String {
        self.context_stack.last().unwrap().name.clone()
    }

    fn enter_context(&mut self, name: String) {
        let var_start = self.context_stack.last().unwrap().allocated;
        let global_start = self.context_stack.first().unwrap().allocated;
        self.context_stack.push(ComponentContext::new(name, var_start, global_start));
    }

    pub fn new_context(&mut self, name: String) -> ContextMarker {
        self.enter_context(name);

        let self_ptr = self as *mut BaseComposer;

        ContextMarker::new(Box::new(move || {
            unsafe {
                let e = &mut *self_ptr as &mut BaseComposer;
                e.exit_context()
            }
        }))
    }

    fn exit_context(&mut self) {
        let mut context = self.context_stack.pop().unwrap();
        let wires_var = format_ident!("wires_{}", context.name);
        let binds = context.input_wires.iter().map(|w| {
            context.format_wire(*w)
        });
        let code = context.code.clone();
        let closure = quote! {
            | #wires_var: &[usize], #( #binds ) ,* | {
                #code
            }
        };
        let key = format!("{}", closure);
        let closure_name = if self.compiled_contexts.contains_key(&key) {
            self.compiled_contexts.get(&key).unwrap().closure_name.clone()
        } else {
            let closure_name = format!("{}_{}", context.name, context.global_start);
            context.closure_name = closure_name.clone();
            let cn = format_ident!("{}", closure_name);
            context.code = quote! {
                let #cn = #closure;
            };
            self.compiled_contexts.insert(key.clone(), context.clone());
            closure_name
        };
        let closure_ident = format_ident!("{}", closure_name);
        if !self.compiled_contexts.contains_key(&key) {
        }
        let prev_context = self.context_stack.last_mut().unwrap();
        let input_wires_iter = context.input_wires.iter().map(|w| {
            prev_context.format_and_allocate_wire(*w)
        });
        let input_wires = quote!( #( #input_wires_iter ) ,* );
        let wires_var = format_ident!("wires_{}", prev_context.name);
        let start = context.var_start;
        let end = context.var_start + context.allocated;

        self.runtime(quote! {
            #closure_ident( #wires_var[#start..#end].try_into().unwrap(), #input_wires );
        });
    }

    pub fn add(&mut self, a: Wire, b: Wire) -> Wire {
        let _e = self.new_context("add".into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    pub fn add_const(&mut self, a: Wire, b: Fp) -> Wire {
        let _e = self.new_context("add_const".into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a + #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    pub fn sub(&mut self, a: Wire, b: Wire) -> Wire {
        let _e = self.new_context("sub".into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    pub fn sub_const(&mut self, a: Wire, b: Fp) -> Wire {
        let _e = self.new_context("sub_const".into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    pub fn mul(&mut self, a: Wire, b: Wire) -> Wire {
        let _e = self.new_context("mul".into());
        // self.new_input(a);
        // self.new_input(b);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
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
        let defs = self.compiled_contexts.iter().map(|(_, c) | {
            c.code.clone()
        });

        let context = self.context_stack.pop().unwrap();
        let n = context.allocated;
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

                #( #defs )*

                #main

                // #intent
            }
        }
    }
}
