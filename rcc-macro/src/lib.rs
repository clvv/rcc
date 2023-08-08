extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::{parse, ItemFn};

#[proc_macro_attribute]
/// ```
///     #[component_of(c)]
///     fn circuit_component(c: &mut Composer, ...)
/// ```
/// Open up a new context that encapsulates generated runtime code within the function.
/// This does not change the behavior of the circuit or witness generation.
/// For frequently-used circuit components, this significnatly speeds up compilation time.
pub fn component_of(composer_var: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse::<ItemFn>(item.clone()).unwrap();
    let name = format!("{}", f.sig.ident);
    let vis = f.vis;
    let sig = f.sig;
    let composer_var = format_ident!("{}", format!("{}", composer_var));

    let stmts = f.block.stmts;

    let marker = format_ident!("__context_marker");

    let body = quote! {
        #vis #sig {

            let #marker = #composer_var.new_context(#name.into());

            #( #stmts ) *
        }
    };

    body.into()
}

