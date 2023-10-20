use quote::quote;
use syn::punctuated::Punctuated;
use syn::{AttrStyle, Attribute, Item, Meta, Token};

use crate::Argument;

fn is_proc(attribute: &Attribute) -> bool {
    if !matches!(attribute.style, AttrStyle::Outer) {
        return false;
    }
    match &attribute.meta {
        Meta::Path(path) => path.segments.last().unwrap().ident == "proc",
        Meta::List(list) => list.path.segments.last().unwrap().ident == "proc",
        _ => false,
    }
}

fn is_module(attribute: &Attribute) -> bool {
    if !matches!(attribute.style, AttrStyle::Outer) {
        return false;
    }
    match &attribute.meta {
        Meta::Path(path) => path.segments.last().unwrap().ident == "module",
        Meta::List(list) => list.path.segments.last().unwrap().ident == "module",
        _ => false,
    }
}

pub(crate) fn impl_attr(
    item: Item,
    options: Punctuated<Argument, Token![,]>,
) -> syn::Result<proc_macro2::TokenStream> {
    let trilogy = options
        .iter()
        .find_map(|arg| arg.crate_name())
        .map(|id| quote! { #id })
        .unwrap_or_else(|| quote! { trilogy });

    let Item::Mod(module) = item else {
        return Err(syn::Error::new_spanned(
            item,
            "this attribute may only be used on mod items",
        ));
    };

    let name = &module.ident;
    let vis = &module.vis;

    let Some(content) = &module.content else {
        return Err(syn::Error::new_spanned(
            module,
            "Trilogy native module must have a body",
        ));
    };

    let items = content.1.iter().filter_map(|item| match item {
        Item::Fn(fn_item) if fn_item.attrs.iter().any(is_proc) => {
            let fn_name = &fn_item.sig.ident;
            Some(quote! {
                module = module.add_procedure(stringify!(#fn_name), #name::#fn_name);
            })
        }
        Item::Mod(module_item) if module_item.attrs.iter().any(is_module) => {
            let child_name = &module_item.ident;
            Some(quote! {
                module = module.add_submodule(#name::#child_name());
            })
        }
        _ => None,
    });

    Ok(quote! {
        #module

        #vis fn #name() -> #trilogy::NativeModule {
            let mut module = #trilogy::NativeModuleBuilder::new();
            #(#items)*
            module.build()
        }
    })
}
