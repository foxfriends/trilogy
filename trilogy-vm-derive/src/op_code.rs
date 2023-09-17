use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Data, DataEnum, DeriveInput, Error, Fields, Ident, LitStr, Token, Type,
};

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(derive);
    syn::custom_keyword!(repr);
    syn::custom_keyword!(doc);
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
    Doc {
        _doc_token: kw::doc,
        _eq_token: Token![=],
        doc: LitStr,
    },
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            Ok(Self::Name {
                _name_token: input.parse::<kw::name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::derive) {
            let content;
            Ok(Self::Derive {
                _derive_token: input.parse::<kw::derive>()?,
                _paren_token: parenthesized!(content in input),
                derives: content.parse_terminated(Ident::parse, Token![,])?,
            })
        } else if lookahead.peek(kw::repr) {
            let content;
            Ok(Self::Repr {
                _repr_token: input.parse::<kw::repr>()?,
                _paren_token: parenthesized!(content in input),
                repr: content.parse()?,
            })
        } else if lookahead.peek(kw::doc) {
            Ok(Self::Doc {
                _doc_token: input.parse::<kw::doc>()?,
                _eq_token: input.parse::<Token![=]>()?,
                doc: input.parse::<LitStr>()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

enum VariantArgument {
    Name {
        _name_token: kw::name,
        _eq_token: Token![=],
        value: LitStr,
    },
}

impl Parse for VariantArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            Ok(Self::Name {
                _name_token: input.parse::<kw::name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn impl_derive(ast: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &ast.ident;
    let vis = &ast.vis;
    let attrs: Vec<Argument> = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("opcode"))
        .map(|attr| {
            attr.parse_args_with(|input: ParseStream| {
                input.parse_terminated(Argument::parse, Token![,])
            })
        })
        .transpose()?
        .into_iter()
        .flatten()
        .collect();
    let name = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Name { value, .. } => Some(value.clone()),
            _ => None,
        })
        .unwrap_or(format_ident!("OpCode"));
    let doc = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Doc { doc, .. } => Some(quote! { #[doc = #doc] }),
            _ => None,
        })
        .into_iter();
    let repr = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Repr { repr, .. } => Some(quote! { #[repr(#repr)] }),
            _ => None,
        })
        .into_iter();
    let derive = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Derive { derives, .. } => {
                let derives = derives.into_iter();
                Some(quote! { #[derive(#(#derives),*)] })
            }
            _ => None,
        })
        .into_iter();

    let mut declarations = vec![];
    let mut params = vec![];
    let mut conversions = vec![];
    let mut from_string = vec![];
    let mut to_string = vec![];

    match ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                let ident = &variant.ident;
                let asm = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("opcode"))
                    .map(|attr| -> syn::Result<_> {
                        let VariantArgument::Name { value, .. } =
                            attr.parse_args::<VariantArgument>()?;
                        Ok(quote!(#value))
                    })
                    .transpose()?
                    .unwrap_or_else(|| {
                        let name = variant.ident.to_string().to_uppercase();
                        quote!(#name)
                    });
                to_string.push(quote! { Self::#ident => #asm });
                from_string.push(quote! { #asm => Self::#ident });

                let attrs = variant
                    .attrs
                    .iter()
                    .filter(|attr| !attr.path().is_ident("opcode"))
                    .map(|attr| -> syn::Result<_> {
                        let tokens = &attr.meta.require_list()?.tokens;
                        Ok(quote! { #[#tokens] })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                declarations.push(quote! { #(#attrs)* #ident });

                match variant.fields {
                    Fields::Unit => {
                        conversions.push(quote! { Self::#ident => #name::#ident });
                        params.push(quote! {Self::#ident => 0 });
                    }
                    Fields::Unnamed(fields) => {
                        conversions.push(quote! { Self::#ident(..)  => #name::#ident });
                        let count = fields.unnamed.len();
                        params.push(quote! { Self::#ident => #count });
                    }
                    field @ Fields::Named(..) => {
                        let error =
                            syn::Error::new_spanned(field, "record variants are not supported")
                                .into_compile_error();
                        conversions.push(quote! { Self::#ident { .. } => #error });
                        params.push(quote! { Self::#ident { .. } => #error });
                    }
                };
            }
        }
        _ => return Err(Error::new_spanned(ast, "only enums are supported")),
    };

    Ok(quote! {
        #(#derive)*
        #(#repr)*
        #(#doc)*
        #vis enum #name {
            #(#declarations),*
        }

        impl #name {
            pub fn params(&self)-> usize {
                match self {
                    #(#params),*
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let token = match self {
                    #(#to_string),*
                };
                token.fmt(f)
            }
        }

        impl std::str::FromStr for #name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    #(#from_string,)*
                    _ => return Err(())
                })
            }
        }

        impl #ident {
            pub fn op_code(&self) -> #name {
                match self {
                    #(#conversions),*
                }
            }
        }
    })
}
