use proc_macro::TokenStream;

mod op_code;

#[proc_macro_derive(OpCode, attributes(opcode))]
pub fn op_code_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    op_code::impl_derive(ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
