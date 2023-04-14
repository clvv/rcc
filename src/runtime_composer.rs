use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use indexmap::IndexMap;
use crate::{Composer, ContextMarker};

#[derive(Debug, Copy, Clone)]
/// A compile time representation of a circuit wire
/// It simply keeps track of its position
pub struct Wire {
    pub global_index: usize,
    /// A hack to make the composer accessible when ToToken is run for the wire
    /// TODO: implement a custom quote macro and remove this
    composer_ptr: *mut RuntimeComposer
}

impl Wire {
    pub fn new(global_index: usize, ptr_composer: *mut RuntimeComposer) -> Wire {
        Wire { global_index, composer_ptr: ptr_composer }
    }

    /// Print out runtime code that access the allocated wire
    pub fn format_against_latest_context(&self) -> TokenStream {
        unsafe {
            let e = &mut *self.composer_ptr as &mut RuntimeComposer;
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
pub struct ComponentContext {
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

/// The RuntimeComposer exposes interfaces to compose runtime witness generation for circuits.
#[derive(Default, Clone, Debug)]
pub struct RuntimeComposer {
    context_stack: Vec<ComponentContext>,
    compiled_contexts: IndexMap<String, ComponentContext>,
}

impl Composer for RuntimeComposer {
    type Wire = Wire;
    type BaseComposer = ();

    /// Enters into a new context and exits automatically when the returned marker is dropped
    fn new_context(&mut self, name: String) -> ContextMarker {
        self.enter_context(name);

        let self_ptr = self as *mut RuntimeComposer;

        ContextMarker::new(Box::new(move || {
            unsafe {
                let e = &mut *self_ptr as &mut RuntimeComposer;
                e.exit_context()
            }
        }))
    }

    /// Generate runtime code into the current context
    fn runtime(&mut self, code: TokenStream) {
        self.context_stack.last_mut().unwrap().code.extend(code.clone());
    }

    fn enter_context(&mut self, name: String) {
        let var_start = self.context_stack.last().unwrap().allocated;
        let global_start = self.context_stack.first().unwrap().allocated;
        self.context_stack.push(ComponentContext::new(name, var_start, global_start));
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

}

impl RuntimeComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.context_stack.push(ComponentContext::new("".into(), 0, 0));
        s
    }

    /// Allocate a new wire and return it
    pub fn new_wire(&mut self) -> Wire {
        let m = self.context_stack.first().unwrap().allocated;
        for context in self.context_stack.iter_mut() {
            context.allocated += 1
        }
        Wire::new(m, self as *mut RuntimeComposer)
    }

    /// Returns a TokenStream encoding a closure that computes all the witnesses
    /// - `prelude` code sets up necessary imports and must set a type `WireVal`
    /// - `init` code has access to a closure `wire: Fn(usize) -> &mut WireVal`
    pub fn compose_rust_witness_gen(&mut self, prelude: TokenStream, init: TokenStream) -> TokenStream {
        let defs = self.compiled_contexts.iter().map(|(_, c) | {
            c.code.clone()
        });

        let context = self.context_stack.pop().unwrap();
        let n = context.allocated;
        let main = context.code;

        quote! {
            || {
                #prelude

                let wires: Vec<WireVal> = vec![WireVal::default(); #n];

                let wire = |i: usize| {
                    unsafe {
                        &mut *(wires.get_unchecked(i) as *const WireVal as *mut WireVal)
                    }
                };

                #init

                #( #defs )*

                let wires_: Vec<usize> = (0..#n).collect();

                #main;

                wires
            }
        }
    }
}
