use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed};

pub(crate) fn impl_derive(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    match &ast.data {
        Data::Struct(DataStruct { fields, .. }) => {
            let include: Vec<_> = match fields {
                Fields::Named(FieldsNamed { named, .. }) => named
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        let name = field.ident.as_ref().unwrap();
                        if i == 0 {
                            quote! {
                                let span = self.#name.span();
                            }
                        } else {
                            quote! {
                                let span = span.union(self.#name.span());
                            }
                        }
                    })
                    .collect(),
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, ..)| {
                        if i == 0 {
                            quote! {
                                let span = self.#i.span();
                            }
                        } else {
                            quote! {
                                let span = span.union(self.#i.span());
                            }
                        }
                    })
                    .collect(),
                Fields::Unit => unimplemented!("unit structs are not used"),
            };
            quote! {
                impl crate::Spanned for #name {
                    fn span(&self) -> source_span::Span {
                        #(#include)*
                        span
                    }
                }
            }
            .into()
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let variants: Vec<_> = variants
                .iter()
                .map(|variant| {
                    let name = &variant.ident;
                    match &variant.fields {
                        Fields::Named(FieldsNamed { named, .. }) => {
                            let names = named.iter().map(|field| {
                                format_ident!("field{}", field.ident.as_ref().unwrap())
                            });
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
                impl crate::Spanned for #name {
                    fn span(&self) -> source_span::Span {
                        match self {
                            #(#variants)*
                        }
                    }
                }
            }
            .into()
        }
        Data::Union(..) => unimplemented!("unions are not used"),
    }
}
