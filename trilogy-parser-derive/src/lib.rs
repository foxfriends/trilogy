use proc_macro::TokenStream;

mod pretty_print_sexpr;
mod spanned;

#[proc_macro_derive(Spanned)]
pub fn spanned_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    spanned::impl_derive(ast)
}

#[proc_macro_derive(PrettyPrintSExpr)]
pub fn pretty_print_sexpr_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    pretty_print_sexpr::impl_derive(ast)
}
