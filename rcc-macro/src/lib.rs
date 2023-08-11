extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::{parse, ItemFn};

#[proc_macro_attribute]
/// ```
///     #[component_of(c)]
///     fn circuit_component(c: &mut Builder, ...)
/// ```
/// Open up a new context that encapsulates generated runtime code within the function.
/// This does not change the behavior of the circuit or witness generation.
/// For frequently-used circuit components, this significnatly speeds up compilation time.
pub fn component_of(builder_var: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse::<ItemFn>(item.clone()).unwrap();
    let name = format!("{}", f.sig.ident);
    let vis = f.vis;
    let sig = f.sig;
    let builder_var = format_ident!("{}", format!("{}", builder_var));

    let stmts = f.block.stmts;

    let marker = format_ident!("__context_marker");

    let body = quote! {
        #vis #sig {

            let #marker = #builder_var.new_context(#name.into());

            #( #stmts ) *
        }
    };

    body.into()
}

#[proc_macro_attribute]
/// ```
///     #[component]
///     fn circuit_component(...)
/// ```
/// Open up a new context that encapsulates generated runtime code within the function.
/// This does not change the behavior of the circuit or witness generation.
/// For frequently-used circuit components, this significnatly speeds up compilation time.
pub fn component(_: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse::<ItemFn>(item.clone()).unwrap();
    let name = format!("{}", f.sig.ident);
    let vis = f.vis;
    let sig = f.sig;
    let stmts = f.block.stmts;

    let marker = format_ident!("__context_marker");

    let body = quote! {
        #vis #sig {

            let #marker = builder().new_context(#name.into());

            #( #stmts ) *
        }
    };

    body.into()
}

#[proc_macro_attribute]
/// ```
///     #[circuit_main]
///     fn my_circuit(e: &mut Builder)
/// ```
/// Marks a function as the main entry function for a circuit.
pub fn circuit_main(_: TokenStream, mut entry: TokenStream) -> TokenStream {
    let f = parse::<ItemFn>(entry.clone()).unwrap();
    let name = f.sig.ident;
    let main = quote! {
        fn main() {
            set_builder();
            #name();
            builder().compile_from_commandline(file!());
        }
    };
    let m: TokenStream = main.into();
    entry.extend(m);
    entry
}
