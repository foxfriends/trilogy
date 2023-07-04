use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Data, DataEnum, DeriveInput, Error, Fields, Ident, Token, Type,
};

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(derive);
    syn::custom_keyword!(repr);
}

enum Argument {
    Name {
        _name_token: kw::name,
        _eq_token: Token![=],
        value: Ident,
    },
    Derive {
        _derive_token: kw::derive,
        _paren_token: token::Paren,
        derives: Punctuated<Ident, Token![,]>,
    },
    Repr {
        _repr_token: kw::repr,
        _paren_token: token::Paren,
        repr: Type,
    },
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            Ok(Argument::Name {
                _name_token: input.parse::<kw::name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::derive) {
            let content;
            Ok(Argument::Derive {
                _derive_token: input.parse::<kw::derive>()?,
                _paren_token: parenthesized!(content in input),
                derives: content.parse_terminated(Ident::parse, Token![,])?,
            })
        } else if lookahead.peek(kw::repr) {
            let content;
            Ok(Argument::Repr {
                _repr_token: input.parse::<kw::repr>()?,
                _paren_token: parenthesized!(content in input),
                repr: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn impl_derive(ast: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &ast.ident;
    let vis = &ast.vis;
    let tags_attr: Vec<Argument> = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("tags"))
        .map(|attr| {
            attr.parse_args_with(|input: ParseStream| {
                input.parse_terminated(Argument::parse, Token![,])
            })
        })
        .transpose()?
        .into_iter()
        .flatten()
        .collect();
    let name = tags_attr
        .iter()
        .find_map(|arg| match arg {
            Argument::Name { value, .. } => Some(value.clone()),
            _ => None,
        })
        .unwrap_or(format_ident!("{ident}Tag"));
    let repr = tags_attr
        .iter()
        .find_map(|arg| match arg {
            Argument::Repr { repr, .. } => Some(quote! { #[repr(#repr)] }),
            _ => None,
        })
        .unwrap_or(quote!());
    let derive = tags_attr
        .iter()
        .find_map(|arg| match arg {
            Argument::Derive { derives, .. } => {
                let derives = derives.into_iter();
                Some(quote! { #[derive(#(#derives),*)] })
            }
            _ => None,
        })
        .unwrap_or(quote! {});

    let mut declarations = vec![];
    let mut conversions = vec![];

    match ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                let attrs = variant
                    .attrs
                    .iter()
                    .filter(|attr| attr.path().is_ident("tags"))
                    .map(|attr| -> syn::Result<_> {
                        let tokens = &attr.meta.require_list()?.tokens;
                        Ok(quote! { #[#tokens] })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let ident = &variant.ident;
                let pattern = match variant.fields {
                    Fields::Unit => quote!(Self::#ident),
                    Fields::Unnamed(..) => quote!(Self::#ident(..)),
                    Fields::Named(..) => quote!(Self::#ident{ .. }),
                };
                declarations.push(quote! { #(#attrs)* #ident });
                conversions.push(quote! { #pattern => #name::#ident })
            }
        }
        _ => return Err(Error::new_spanned(ast, "only enums are supported")),
    };

    Ok(quote! {
        #derive
        #repr
        #vis enum #name {
            #(#declarations),*
        }

        impl crate::traits::Tags for #ident {
            type Tag = #name;

            fn tag(&self) -> Self::Tag {
                match self {
                    #(#conversions),*
                }
            }
        }
    })
}
