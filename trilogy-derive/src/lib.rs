//! Macros to bridge the gap between Rust and Trilogy.
//!
//! These macros are provided to safely bridge Rust types, modules, and functions
//! to the Trilogy runtime.
//!
//! They are actually attribute macros, not derive macros, so this crate would
//! be better named just `trilogy_macros`.
//!
//! This crate is distributed alongside Trilogy under the feature `macros`,
//! so it is not likely you will need to install it manually.

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

/// Constructs a Trilogy native function out of a Rust function.
///
/// This is the only safe way to implement the `NativeFunction` trait for your
/// own functions. The result is a procedure that can be called from Trilogy,
/// as if it were any other Trilogy procedure.
#[proc_macro_attribute]
pub fn proc(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let args: Punctuated<Argument, Token![,]> = Punctuated::parse_terminated.parse(attr).unwrap();
    proc::impl_attr(item, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Constructs a Trilogy native module out of a Rust module.
///
/// Functions within this module marked with the `#[proc]` attribute are exported as
/// procedures which may be called from the Trilogy program.
///
/// The resulting value is a NativeModule that can be installed as a library into the
/// Trilogy runtime via the builder API.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let args: Punctuated<Argument, Token![,]> = Punctuated::parse_terminated.parse(attr).unwrap();
    module::impl_attr(item, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
