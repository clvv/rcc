extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::{parse, ItemFn};

/// `component(e)` where `e` is the composer
#[proc_macro_attribute]
pub fn component(composer_var: TokenStream, item: TokenStream) -> TokenStream {
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
