extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse, ItemFn};

fn prepend_code_to_function(code: proc_macro2::TokenStream, f: TokenStream) -> TokenStream {
    let f = parse::<ItemFn>(f.clone()).unwrap();
    let vis = f.vis;
    let sig = f.sig;
    let stmts = f.block.stmts;

    let body = quote! {
        #vis #sig {
            #code

            #( #stmts ) *
        }
    };
    body.into()
}

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
    let marker = format_ident!("__context_marker");
    let builder_var = format_ident!("{}", format!("{}", builder_var));

    let code = quote! {
        let #marker = #builder_var.new_context(#name.into());
    };

    prepend_code_to_function(code, item)
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
    let marker = format_ident!("__context_marker");

    let code = quote! {
        let #marker = builder().new_context(#name.into());
    };

    prepend_code_to_function(code, item)
}

#[proc_macro_attribute]
/// ```
///     #[main_component]
///     fn my_circuit()
/// ```
/// Marks a function as the main entry function for a circuit.
pub fn main_component(_: TokenStream, entry: TokenStream) -> TokenStream {
    let f = parse::<ItemFn>(entry.clone()).unwrap();
    let name = format!("{}", f.sig.ident);
    let name_ident = f.sig.ident;
    let marker = format_ident!("__context_marker");

    let code = quote! {
        let #marker = builder().new_context(#name.into());
    };

    let mut f = prepend_code_to_function(code, entry);

    let main = quote! {
        fn main() {
            set_builder();
            #name_ident();
            builder().compile_from_commandline(file!());
        }
    };
    let m: TokenStream = main.into();

    f.extend(m);
    f
}
