use proc_macro2::TokenStream;

pub mod runtime_composer;
pub mod mock_composer;

/// Composer trait
pub trait Composer {
    type Wire;
    type ContextMarker;

    fn enter_context(&mut self, name: String);
    fn exit_context(&mut self);
    fn new_context(&mut self, name: String) -> Self::ContextMarker;
    fn runtime(&mut self, code: TokenStream);
}
