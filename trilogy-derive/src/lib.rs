use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Path, Token};

mod module;
mod proc;

mod kw {
    syn::custom_keyword!(crate_name);
}

pub(crate) enum Argument {
    CrateName {
        _crate_token: kw::crate_name,
        _eq_token: Token![=],
        value: Path,
    },
}

impl Argument {
    fn crate_name(&self) -> Option<&Path> {
        match self {
            Self::CrateName { value, .. } => Some(value),
        }
    }
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::crate_name) {
            Ok(Argument::CrateName {
                _crate_token: input.parse::<kw::crate_name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[proc_macro_attribute]
pub fn proc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let args: Punctuated<Argument, Token![,]> = Punctuated::parse_terminated.parse(attr).unwrap();
    proc::impl_attr(item, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let args: Punctuated<Argument, Token![,]> = Punctuated::parse_terminated.parse(attr).unwrap();
    module::impl_attr(item, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
