use proc_macro::TokenStream;

mod spanned;

#[proc_macro_derive(Spanned)]
pub fn spanned_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    spanned::impl_derive(ast)
}
