use proc_macro::TokenStream;

mod proc;

#[proc_macro_attribute]
pub fn proc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    proc::impl_attr(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
