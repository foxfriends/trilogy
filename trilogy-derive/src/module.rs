use quote::quote;
use syn::punctuated::Punctuated;
use syn::{AttrStyle, Attribute, Item, ItemImpl, ItemMod, Meta, Token};

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

fn is_func(attribute: &Attribute) -> bool {
    if !matches!(attribute.style, AttrStyle::Outer) {
        return false;
    }
    match &attribute.meta {
        Meta::Path(path) => path.segments.last().unwrap().ident == "func",
        Meta::List(list) => list.path.segments.last().unwrap().ident == "func",
        _ => false,
    }
}

type Options = Punctuated<Argument, Token![,]>;

pub(crate) fn impl_attr(item: Item, options: Options) -> syn::Result<proc_macro2::TokenStream> {
    match item {
        Item::Mod(module) => impl_module(module, options),
        Item::Impl(module) => impl_impl(module, options),
        _ => {
            return Err(syn::Error::new_spanned(
                item,
                "the `#[module]` attribute may only be used on module, struct, or impl items",
            ));
        }
    }
}

fn impl_impl(module: ItemImpl, options: Options) -> syn::Result<proc_macro2::TokenStream> {
    let self_ty = &module.self_ty;
    let trilogy = options
        .iter()
        .find_map(|arg| arg.crate_name())
        .map(|id| quote! { #id })
        .unwrap_or_else(|| quote! { trilogy });
    let trilogy_vm = options
        .iter()
        .find_map(|arg| arg.vm_crate_name())
        .map(|id| quote! { #id })
        .unwrap_or_else(|| quote! { trilogy_vm });

    // let items = module.items.iter().filter_map(|item| match item {
    //     ImplItem::Fn(fn_item)
    //         if fn_item
    //             .attrs
    //             .iter()
    //             .any(|item| is_proc(item) || is_func(item)) =>
    //     {
    //         let fn_name = &fn_item.sig.ident;
    //         let self_param = fn_item.sig.receiver().map(|_| quote! { self, });
    //         fn_item
    //             .sig
    //             .inputs
    //             .iter()
    //             .skip(if self_param.is_some() { 1 } else { 0 })
    //             .map(|input| match input {
    //                 FnArg::Typed(param) => quote!(inputs.next().unwrap()),
    //                 _ => unreachable!(),
    //             });
    //         Some(quote! {
    //             if atom == runtime.atom(stringify!(fn_name)) {
    //                 return fn_name(#self_param runtime, #(#inputs),*);
    //             }
    //         })
    //     }
    //     _ => None,
    // });

    Ok(quote! {
        #module

        impl #trilogy::NativeFunction for #self_ty {
            fn arity(&self) -> usize {
                2 // the symbol + the module key
            }

            fn call(&mut self, runtime: &mut #trilogy_vm::Execution, mut input: Vec<#trilogy_vm::Value>) -> std::result::Result<(), #trilogy_vm::Error> {
                let runtime = #trilogy::Runtime::new(runtime);
                let atom = runtime.unlock_module(input)?;

                // #(#items)*

                // let symbol_list = self
                //     .items
                //     .keys()
                //     .map(|name| Value::from(runtime.atom(name)))
                //     .collect::<Vec<_>>();
                Err(runtime.unresolved_import(atom, vec![]))
            }
        }
    })
}

fn impl_module(module: ItemMod, options: Options) -> syn::Result<proc_macro2::TokenStream> {
    let trilogy = options
        .iter()
        .find_map(|arg| arg.crate_name())
        .map(|id| quote! { #id })
        .unwrap_or_else(|| quote! { trilogy });

    let name = &module.ident;
    let vis = &module.vis;

    let Some(content) = &module.content else {
        return Err(syn::Error::new_spanned(
            module,
            "Trilogy native module must have a body",
        ));
    };

    let items = content.1.iter().filter_map(|item| match item {
        Item::Fn(fn_item)
            if fn_item
                .attrs
                .iter()
                .any(|attr| is_proc(attr) || is_func(attr)) =>
        {
            let fn_name = &fn_item.sig.ident;
            Some(quote! {
                module = module.add_item(stringify!(#fn_name), #name::#fn_name);
            })
        }
        Item::Struct(struct_item)
            if struct_item
                .attrs
                .iter()
                .any(|attr| is_proc(attr) || is_func(attr)) =>
        {
            let struct_name = &struct_item.ident;
            Some(quote! {
                module = module.add_item(stringify!(#struct_name), #name::#struct_name);
            })
        }
        Item::Mod(module_item) if module_item.attrs.iter().any(is_module) => {
            let child_name = &module_item.ident;
            Some(quote! {
                module = module.add_item(stringify!(#child_name), #name::#child_name());
            })
        }
        _ => None,
    });

    Ok(quote! {
        #module

        #[doc(hidden)]
        #vis fn #name() -> #trilogy::NativeModule {
            let mut module = #trilogy::NativeModuleBuilder::new();
            #(#items)*
            module.build()
        }
    })
}
