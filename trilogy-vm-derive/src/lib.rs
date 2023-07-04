use proc_macro::TokenStream;

mod tags;

#[proc_macro_derive(Tags, attributes(tags))]
pub fn tags_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    tags::impl_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
