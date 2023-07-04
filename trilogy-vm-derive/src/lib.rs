use proc_macro::TokenStream;

mod asm;
mod tags;

#[proc_macro_derive(Tags, attributes(tags))]
pub fn tags_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    tags::impl_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Asm, attributes(asm))]
pub fn asm_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    asm::impl_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
