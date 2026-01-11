use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

pub(crate) fn impl_derive(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let generic_params = if ast.generics.lt_token.is_some() {
        let params = ast.generics.params.iter();
        quote! { <#(#params: crate::Spanned),*> }
    } else {
        quote!()
    };
    match &ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            let variants: Vec<_> = variants
                .iter()
                .map(|variant| {
                    let name = &variant.ident;
                    match &variant.fields {
                        Fields::Named(FieldsNamed { named, .. }) => {
                            let names = named
                                .iter()
                                .map(|field| format_ident!("{}", field.ident.as_ref().unwrap()));
                            let include = named.iter().enumerate().map(|(i, field)| {
                                let name = field.ident.as_ref().unwrap();
                                if i == 0 {
                                    quote! {
                                        let span = #name.span();
                                    }
                                } else {
                                    quote! {
                                        let span = span.union(#name.span());
                                    }
                                }
                            });
                            quote! {
                                Self::#name { #(#names),* } => {
                                    #(#include)*
                                    span
                                }
                            }
                        }
                        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            let names: Vec<_> = unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, ..)| format_ident!("field{}", i))
                                .collect();
                            let include = names.iter().enumerate().map(|(i, name)| {
                                if i == 0 {
                                    quote! {
                                        let span = #name.span();
                                    }
                                } else {
                                    quote! {
                                        let span = span.union(#name.span());
                                    }
                                }
                            });
                            quote! {
                                Self::#name ( #(#names),* ) => {
                                    #(#include)*
                                    span
                                }
                            }
                        }
                        Fields::Unit => unimplemented!("unit structs are not used"),
                    }
                })
                .collect();
            quote! {
                impl #generic_params crate::Spanned for #name #generics {
                    fn span(&self) -> source_span::Span {
                        match self {
                            #(#variants)*
                        }
                    }
                }
            }
            .into()
        }
        Data::Union(..) | Data::Struct(..) => {
            unimplemented!("only enums can have Spanned derived")
        }
    }
}
