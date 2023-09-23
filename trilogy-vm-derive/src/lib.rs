//! Internal derive macros for the Trilogy virtual machine.
//!
//! This crate is not intended for external usage.

use proc_macro::TokenStream;

mod asm;

#[proc_macro_derive(Asm, attributes(asm))]
pub fn asm_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    asm::impl_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
