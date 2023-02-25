use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
// use std::collections::HashMap;
use indexmap::IndexMap;

pub use ark_ff::{BigInteger, BigInt, Field, PrimeField};
pub use ark_bn254::Fr as F;

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

    /// Print out runtime code that access the allocated wire
    fn format_against_latest_context(&self) -> TokenStream {
        unsafe {
            let e = &mut *self.ptr_composer as &mut BaseComposer;
            let last_context = e.context_stack.last_mut().unwrap();
            let id = last_context.format_and_mark_input(*self);
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
    /// Number of all new wires that gets allocated inside the component
    allocated: usize,
    /// Allocation index relative to the previous context
    var_start: usize,
    /// Allocation index relative to the global context
    global_start: usize,
    /// List of accessed wires that are not allocated in this context
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

    /// Determine if a given wire has been marked as an input wire for this context
    /// If so, return the input index
    fn input_index(&self, w: Wire) -> Option<usize> {
        self.input_wires.iter().enumerate().find_map(|(i, a)| {
            if a.global_index == w.global_index {
                Some(i)
            } else { None }
        })
    }

    /// Print out runtime code accessing the allocated wire
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

    /// Similar to above, but marks a wire as input if it has not been marked so
    fn format_and_mark_input(&mut self, w: Wire) -> TokenStream {
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

/// A hack to keep automatically call a function when a Rust context exits
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
    compiled_contexts: IndexMap<String, ComponentContext>,
    constants: IndexMap<String, Wire>
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

    /// Allocated a new wire and return it
    pub fn new_wire(&mut self) -> Wire {
        let m = self.context_stack.first().unwrap().allocated;
        for context in self.context_stack.iter_mut() {
            context.allocated += 1
        }
        // TODO: actually allocated a wire inside the constraint system
        Wire::new(m, self as *mut BaseComposer)
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
            let m = self.context_stack.first().unwrap().allocated;
            for context in self.context_stack.iter_mut() {
                context.allocated += 1
            }
            let w = Wire::new(m, self as *mut BaseComposer);
            self.constants.insert(key, w);
            w
        }
        // TODO: actually allocated a constant wire in the constraint system
    }

    /// Generate runtime code into the current context
    pub fn runtime(&mut self, code: TokenStream) {
        self.context_stack.last_mut().unwrap().code.extend(code.clone());
    }

    pub fn get_latest_context_name(&self) -> String {
        self.context_stack.last().unwrap().name.clone()
    }

    fn enter_context(&mut self, name: String) {
        let var_start = self.context_stack.last().unwrap().allocated;
        let global_start = self.context_stack.first().unwrap().allocated;
        self.context_stack.push(ComponentContext::new(name, var_start, global_start));
    }

    /// Enters into a new context and exits automatically when the returned marker is dropped
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

    /// Exits a context
    /// Compile the compute closure for the context
    /// Invokes the compute closure
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

        // Check to see if a compatible closure has been compiled before
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
        let prev_context = self.context_stack.last_mut().unwrap();
        let input_wires_iter = context.input_wires.iter().map(|w| {
            prev_context.format_and_mark_input(*w)
        });
        let input_wires = quote!( #( #input_wires_iter ) ,* );
        let wires_var = format_ident!("wires_{}", prev_context.name);
        let start = context.var_start;
        let end = context.var_start + context.allocated;

        // invoke the compute closure
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

        let b = self.new_constant_wire(b.value);

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

        let b = self.new_constant_wire(b.value);

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a - #b;
        });

        // TODO: constraints need to be generated here
        c
    }

    pub fn mul(&mut self, a: Wire, b: Wire) -> Wire {
        let _e = self.new_context("mul".into());

        let c = self.new_wire();
        self.runtime(quote! {
            #c = #a * #b;
        });
        // TODO: constraints need to be generated here
        c
    }

    pub fn sum(&mut self, wires: Vec<Wire>) -> Wire {
        let _e = self.new_context("sum".into());

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
    pub fn runtime_log(&mut self, wire: Wire) {
        self.runtime(quote! {
            println!("{}", #wire.into_bigint());
        });
    }

    pub fn compose_final_circuit(&mut self) -> TokenStream {
        let defs = self.compiled_contexts.iter().map(|(_, c) | {
            c.code.clone()
        });

        let (constant_values, constant_indices): (Vec<_>, Vec<_>) = self.constants.iter().map(|(v, w)| {
            (v, w.global_index)
        }).unzip();

        let constant_decl = quote! {
            #( (*wire(#constant_indices)) = F::from(BigInt!(#constant_values)) ; ) *
        };

        let context = self.context_stack.pop().unwrap();
        let n = context.allocated;
        let main = context.code;

        quote! {
            use rcc::{F, BigInt, PrimeField};
            use std::env;

            fn main() {
                let args: Vec<String> = env::args().collect();
                let wires: Vec<F> = vec![F::default(); #n];

                let wire = |i: usize| {
                    unsafe {
                        &mut *(wires.get_unchecked(i) as *const F as *mut F)
                    }
                };

                #constant_decl

                #( #defs )*

                let wires_: Vec<usize> = (0..#n).collect();
                #main
            }
        }
    }
}
