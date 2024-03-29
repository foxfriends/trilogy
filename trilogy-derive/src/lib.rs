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
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Attribute, LitStr, Path, Token};

mod func;
mod module;
mod proc;

mod kw {
    syn::custom_keyword!(crate_name);
    syn::custom_keyword!(vm_crate_name);
    syn::custom_keyword!(name);
}

pub(crate) enum Argument {
    CrateName {
        _crate_token: kw::crate_name,
        _eq_token: Token![=],
        value: Path,
    },
    VmCrateName {
        _crate_token: kw::vm_crate_name,
        _eq_token: Token![=],
        value: Path,
    },
    Name {
        _crate_token: kw::name,
        _eq_token: Token![=],
        value: LitStr,
    },
}

impl Argument {
    fn crate_name(&self) -> Option<&Path> {
        match self {
            Self::CrateName { value, .. } => Some(value),
            _ => None,
        }
    }

    fn vm_crate_name(&self) -> Option<&Path> {
        match self {
            Self::VmCrateName { value, .. } => Some(value),
            _ => None,
        }
    }

    fn name(&self) -> Option<String> {
        match self {
            Self::Name { value, .. } => Some(value.value()),
            _ => None,
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
        } else if lookahead.peek(kw::vm_crate_name) {
            Ok(Argument::VmCrateName {
                _crate_token: input.parse::<kw::vm_crate_name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::name) {
            Ok(Argument::Name {
                _crate_token: input.parse::<kw::name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

struct Options {
    args: Punctuated<Argument, Token![,]>,
}

impl Options {
    fn trilogy(&self) -> proc_macro2::TokenStream {
        self.args
            .iter()
            .find_map(|arg| arg.crate_name())
            .map(|id| quote! { #id })
            .unwrap_or_else(|| quote! { trilogy })
    }

    fn trilogy_vm(&self) -> proc_macro2::TokenStream {
        self.args
            .iter()
            .find_map(|arg| arg.vm_crate_name())
            .map(|id| quote! { #id })
            .unwrap_or_else(|| quote! { trilogy_vm })
    }

    fn name(&self, ident: &syn::Ident) -> proc_macro2::TokenStream {
        self.args
            .iter()
            .find_map(|arg| arg.name())
            .map(|id| quote! { #id })
            .unwrap_or_else(|| quote! { stringify!(#ident) })
    }
}

impl TryFrom<Attribute> for Options {
    type Error = syn::Error;

    fn try_from(value: Attribute) -> Result<Self, Self::Error> {
        Ok(Self {
            args: value
                .meta
                .require_list()?
                .parse_args_with(Punctuated::parse_terminated)?,
        })
    }
}

impl TryFrom<TokenStream> for Options {
    type Error = syn::Error;

    fn try_from(value: TokenStream) -> Result<Self, Self::Error> {
        let args = Punctuated::parse_terminated.parse(value)?;
        Ok(Self { args })
    }
}

/// Constructs a Trilogy native procedure out of a Rust function.
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
/// Functions within this module marked with the `#[proc]` or `#[func]` attributes are exported as
/// procedures or functions, respectively, which may be called from the Trilogy program.
///
/// The resulting value is a `NativeModule` that can be installed as a library into the
/// Trilogy runtime via the builder API.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let args = Options::try_from(attr).unwrap();
    module::impl_attr(item, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Constructs a Trilogy native function out of a Rust function.
///
/// This is the only safe way to implement the `NativeFunction` trait for your
/// own functions. The result is a curried function that can be called from Trilogy,
/// as if it were any other Trilogy function.
#[proc_macro_attribute]
pub fn func(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse(item).unwrap();
    let args: Punctuated<Argument, Token![,]> = Punctuated::parse_terminated.parse(attr).unwrap();
    func::impl_attr(item, args)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
