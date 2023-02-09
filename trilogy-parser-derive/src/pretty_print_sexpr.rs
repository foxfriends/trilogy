use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Visibility,
};

pub(crate) fn impl_derive(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    match &ast.data {
        Data::Struct(DataStruct { fields, .. }) => {
            let fields: Vec<_> = match fields {
                Fields::Named(FieldsNamed { named, .. }) => named
                    .iter()
                    .filter(|field| !matches!(field.vis, Visibility::Inherited))
                    .map(|field| field.ident.as_ref().unwrap())
                    .map(|name| quote!(self.#name))
                    .collect(),
                Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| !matches!(field.vis, Visibility::Inherited))
                    .map(|(i, ..)| quote!(self.#i))
                    .collect(),
                Fields::Unit => unimplemented!("unit structs are not used"),
            };
            let name_str = format!("{name}");
            quote! {
                impl<'a> crate::PrettyPrintSExpr<'a> for #name {
                    fn pretty_print_sexpr(&self, printer: &'a crate::PrettyPrinter) -> crate::PrettyPrinted<'a> {
                        use pretty::DocAllocator;
                        let doc = printer.text(#name_str);
                        #(let doc = doc.append(printer.line()).append(#fields.pretty_print_sexpr(printer));)*
                        doc.nest(2).group().parens()
                    }
                }
            }
            .into()
        }
        Data::Enum(DataEnum { variants, .. }) => {
            let variants: Vec<_> = variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let name_str = format!("{name}::{variant_name}");
                    match &variant.fields {
                        Fields::Named(FieldsNamed { named, .. }) => {
                            let names: Vec<_> = named
                                .iter()
                                .map(|field| format_ident!("{}", field.ident.as_ref().unwrap()))
                                .collect();
                            quote! {
                                Self::#variant_name { #(#names),* } => {
                                    let doc = printer.text(#name_str);
                                    #(let doc = doc.append(printer.line()).append(#names.pretty_print_sexpr(printer));)*
                                    doc.nest(2).group().parens()
                                }
                            }
                        }
                        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                            let names: Vec<_> = unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, ..)| format_ident!("field{}", i))
                                .collect();
                            quote! {
                                Self::#variant_name(#(#names),*) => {
                                    let doc = printer.text(#name_str);
                                    #(let doc = doc.append(printer.line()).append(#names.pretty_print_sexpr(printer));)*
                                    doc.nest(2).group().parens()
                                }
                            }
                        }
                        Fields::Unit => quote! {
                            Self::#variant_name => {
                                printer.nil()
                            }
                        },
                    }
                })
                .collect();
            quote! {
                impl<'a> crate::PrettyPrintSExpr<'a> for #name {
                    fn pretty_print_sexpr(&self, printer: &'a crate::PrettyPrinter) -> crate::PrettyPrinted<'a> {
                        use pretty::DocAllocator;
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
