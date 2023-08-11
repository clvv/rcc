use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use indexmap::IndexMap;
use crate::{Composer, ContextMarker, Wire};

#[derive(Debug, Copy, Clone)]
/// A compile time representation of a circuit wire
/// It simply keeps track of its position
pub struct RuntimeWire {
    pub global_id: usize,
    /// A hack to make the composer accessible when ToToken is run for the wire
    /// TODO: implement a custom quote macro and remove this
    composer_ptr: *mut RuntimeComposer
}

impl Wire for RuntimeWire {
    type Composer = RuntimeComposer;

    fn composer(&self) -> &mut RuntimeComposer {
        unsafe {
            &mut *self.composer_ptr as &mut RuntimeComposer
        }
    }
}

impl RuntimeWire {
    /// Print out runtime code that access the allocated wire
    pub fn format_against_latest_context(&self) -> TokenStream {
        let e = self.composer();
        let last_context = e.context_stack.last_mut().unwrap();
        let id = last_context.format_and_mark_input(*self);
        quote! { (*wire(#id)) }
    }
}

/// A compile-time wire is translated into runtime code via this trait
impl ToTokens for RuntimeWire {
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
    input_wires: Vec<RuntimeWire>,
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
    fn input_index(&self, w: RuntimeWire) -> Option<usize> {
        self.input_wires.iter().enumerate().find_map(|(i, a)| {
            if a.global_id == w.global_id {
                Some(i)
            } else { None }
        })
    }

    // /// Print out runtime code accessing the allocated wire
    // fn format_wire(&self, w: RuntimeWire) -> TokenStream {
    //     if let Some(index) = self.input_index(w) {
    //         let id = quote!( input[#index] );
    //         quote! { #id }
    //     } else {
    //         let wires_var = format_ident!("wires_{}", self.name);
    //         let id = w.global_id - self.global_start;
    //         quote! {
    //             #wires_var[#id]
    //         }
    //     }
    // }

    /// Similar to above, but marks a wire as input if it has not been marked so
    fn format_and_mark_input(&mut self, w: RuntimeWire) -> TokenStream {
        if self.global_start > w.global_id {
            // must be an input wire
            if self.input_index(w) == None {
                self.input_wires.push(w);
            }
            let index = self.input_index(w).unwrap();
            let id = quote!( input[#index] );
            quote! { #id }
        } else {
            let wires_var = format_ident!("wires_{}", self.name);
            let id = w.global_id - self.global_start;
            quote! {
                #wires_var[#id]
            }
        }
    }

    //fn format_and_mark_inputs(&mut self, ws: Vec<RuntimeWire>) -> TokenStream {
    //    let mut step;
    //    let mut start = None;
    //    let mut end = None;
    //    ws.iter().for_each(|w| {
    //        match (start, step) {
    //            (None, None) => {
    //                // Have not found the starting input wire
    //                if self.global_start > w.global_id {
    //                    // Input wire
    //                    start = Some(self.input_index(w));
    //                }
    //            },
    //            (Some(i), None) => {
    //                if self.global_start > w.global_id {
    //                    step = w.global_id - i
    //                }
    //            }
    //        }
    //    })
    //    while let n = ws.next() {
    //        if self.global_start <= w.global_id {

    //        }
    //    }
    //}
}

#[derive(Clone, Debug)]
pub enum InputWireType {
    Singleton(RuntimeWire),
    Vector(Vec<RuntimeWire>)
}

/// The RuntimeComposer is a helper that helps the ciruict builder to mange
/// (1) allocation of witnesses (called wires)
/// (2) runtime code that generates the above allocated witnesses from a small set input wires
#[derive(Default, Clone, Debug)]
pub struct RuntimeComposer {
    context_stack: Vec<ComponentContext>,
    compiled_contexts: IndexMap<String, ComponentContext>,
    pub wires: Vec<RuntimeWire>,
    input_wires: IndexMap<String, InputWireType>,
    public_wires: IndexMap<String, RuntimeWire>
}

impl Composer for RuntimeComposer {
    type Wire = RuntimeWire;
    type BaseComposer = ();

    /// Allocate a new wire to a column and return it
    fn new_wire(&mut self) -> RuntimeWire {
        let m = self.context_stack.first().unwrap().allocated;
        for context in self.context_stack.iter_mut() {
            context.allocated += 1;
        }
        let w = RuntimeWire {
            global_id: m,
            composer_ptr: self as *mut RuntimeComposer
        };
        self.wires.push(w);
        w
    }

    /// Register a wire as an input wire
    fn input_wire(&mut self, name: &str) -> RuntimeWire {
        assert!(!self.input_wires.contains_key(name), "Cannot declare the same input wire twice.");
        let w = self.new_wire();
        self.input_wires.insert(name.into(), InputWireType::Singleton(w));
        w
    }

    /// Register a wire as an input wire
    fn input_wires(&mut self, name: &str, n: usize) -> Vec<RuntimeWire> {
        assert!(!self.input_wires.contains_key(name), "Cannot declare the same input wire twice.");
        let ws = self.new_wires(n);
        self.input_wires.insert(name.into(), InputWireType::Vector(ws.clone()));
        ws
    }

    /// The runtime composer simply ignores this request here
    fn declare_public(&mut self, w: RuntimeWire, name: &str) {
        self.public_wires.insert(name.into(), w);
    }

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
        let code = context.code.clone();
        let closure = quote! {
            | #wires_var: &[usize], input: &[usize] | {
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
        let input_wires = quote!( &[#( #input_wires_iter ) ,*] );
        let wires_var = format_ident!("wires_{}", prev_context.name);
        let start = context.var_start;
        let end = context.var_start + context.allocated;

        // invoke the compute closure
        self.runtime(quote! {
            #closure_ident( &#wires_var[#start..#end], #input_wires );
        });
    }

}

impl RuntimeComposer {
    pub fn new() -> Self {
        let mut s = Self::default();
        s.context_stack.push(ComponentContext::new("".into(), 0, 0));
        s
    }

    /// Returns a TokenStream encoding a closure that computes all the witnesses
    /// - `prelude` code sets up necessary imports and must set a type `WireVal`
    pub fn compose_rust_witness_gen(&mut self, prelude: TokenStream, init: TokenStream) -> String {
        let defs = self.compiled_contexts.iter().map(|(_, c) | {
            c.code.clone()
        });

        let n = self.wires.len();
        let main = self.context_stack.pop().unwrap().code;

        let set_input_wires = self.input_wires.iter().map(|(ref key, ref input)| {
            match input {
                InputWireType::Singleton(w) => {
                    let id = w.global_id;
                    quote!( *wire(#id) = *inputs.get(#key).unwrap(); )
                },
                InputWireType::Vector(ws) => {
                    let mut ts = quote!( #( let vs = inputs.get(#key).unwrap(); ));
                    for (i, w) in ws.iter().enumerate() {
                        let id = w.global_id;
                        ts.extend( quote!( *wire(#id) = *vs[#i]; ) );
                    }
                    ts
                }
            }
        });

        let set_public_wires = self.public_wires.iter().map(|(key, wire)| {
            let id = wire.global_id;
            quote!( public.insert(#key.into(), *wire(#id)); )
        });

        let code = quote! {
            #prelude

            use std::collections::HashMap;
            type Input = HashMap<String, F>;
            type Public = HashMap<String, F>;

            pub fn generate_witnesses(inputs: Input) -> (AllWires, Public) {
                #init

                #( #set_input_wires )* ;

                #( #defs )*

                let wires_: &[usize] = &(0..#n).collect::<Vec<_>>();
                #main;

                let mut public: Public = HashMap::new();

                #( #set_public_wires )* ;

                (wires, public)
            }
        };
        let parsed: syn::File = syn::parse2(code).unwrap();
        let formatted = prettyplease::unparse(&parsed);
        formatted
    }
}
