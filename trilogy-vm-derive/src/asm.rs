use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Data, DataEnum, DeriveInput, Error, Fields, FieldsUnnamed, LitStr, Token,
};

mod kw {
    syn::custom_keyword!(name);
}

enum Argument {
    Name {
        _name_token: kw::name,
        _eq_token: Token![=],
        value: LitStr,
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
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn impl_derive(ast: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &ast.ident;
    let mut to_asm = vec![];
    let mut from_asm = vec![];

    match ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                let name = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("asm"))
                    .map(|attr| -> syn::Result<_> {
                        let Argument::Name { value, .. } = attr.parse_args::<Argument>()?;
                        Ok(quote!(#value))
                    })
                    .transpose()?
                    .unwrap_or_else(|| {
                        let name = variant.ident.to_string().to_uppercase();
                        quote!(#name)
                    });
                let ident = &variant.ident;
                to_asm.push(match &variant.fields {
                    Fields::Unit => quote! {
                        Self::#ident => { write!(f, #name)?; }
                    },
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        let fields = unnamed
                            .into_iter()
                            .enumerate()
                            .map(|(i, _)| format_ident!("f{i}"))
                            .collect::<Vec<_>>();
                        quote! {
                            Self::#ident(#(#fields),*) => {
                                write!(f, #name)?;
                                write!(f, " ")?;
                                #(write!(f, "{}", #fields)?;)*
                            }
                        }
                    }
                    Fields::Named(named) => {
                        return Err(Error::new(named.span(), "Named variants are not supported"))
                    }
                });
                from_asm.push(match &variant.fields {
                    Fields::Unit => quote! {
                        #name => { Ok(Self::#ident) }
                    },
                    Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                        if unnamed.len() != 1 {
                            return Err(Error::new(
                                unnamed.span(),
                                "Only a single unnamed parameter is supported",
                            ));
                        }
                        quote! {
                            #name => {
                                Ok(Self::#ident(ctx.parse_param(param)?))
                            }
                        }
                    }
                    Fields::Named(named) => {
                        return Err(Error::new(named.span(), "Named variants are not supported"))
                    }
                });
            }
        }
        _ => return Err(Error::new_spanned(ast, "only enums are supported")),
    };

    Ok(quote! {
        impl crate::bytecode::asm::Asm for #ident {
            fn fmt_asm(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#to_asm)*
                }
                Ok(())
            }

            fn parse_asm(src: &str, ctx: &mut crate::bytecode::asm::AsmContext) -> Result<Self, crate::bytecode::asm::ErrorKind> {
                let (opcode, param) = src
                    .split_once(' ')
                    .map(|(opcode, param)| (opcode, Some(param)))
                    .unwrap_or((src, None));
                match opcode {
                    #(#from_asm)*
                    s => Err(crate::bytecode::asm::ErrorKind::UnknownOpcode(s.to_owned())),
                }
            }
        }
    })
}
