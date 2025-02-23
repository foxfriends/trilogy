use quote::{ToTokens, format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Fields, FnArg, Item, ItemFn, ItemStruct, Token};

use crate::Argument;

type Options = Punctuated<Argument, Token![,]>;

pub(crate) fn impl_attr(item: Item, options: Options) -> syn::Result<proc_macro2::TokenStream> {
    match item {
        Item::Struct(item) => impl_struct(item, options),
        Item::Fn(item) => impl_fn(item, options),
        _ => Err(syn::Error::new_spanned(
            item,
            "the `#[func]` attribute may only be used on struct or fn items",
        )),
    }
}

fn impl_struct(module: ItemStruct, options: Options) -> syn::Result<proc_macro2::TokenStream> {
    let name = &module.ident;
    let vis = &module.vis;

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
    let constructor_name = options
        .iter()
        .find_map(|arg| arg.name())
        .map(|id| quote! { #id })
        .unwrap_or_else(|| format_ident!("{name}Constructor").into_token_stream());

    let Fields::Unnamed(fields) = &module.fields else {
        return Err(syn::Error::new_spanned(
            module.fields,
            "the `#[module]` attribute may only be used on tuple structs",
        ));
    };
    if fields.unnamed.is_empty() {
        return Err(syn::Error::new_spanned(
            module.fields,
            "the `#[module]` attribute may only be used on structs with at least one field. For a module with no parameters, try a `mod` item",
        ));
    }

    let arity = fields.unnamed.len();

    let inputs = fields
        .unnamed
        .iter()
        .map(|_| quote! { input.next().unwrap() });
    Ok(quote! {
        #module

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #constructor_name;

        impl #trilogy::NativeFunction for #constructor_name {
            fn call(&mut self, runtime: &mut #trilogy_vm::Execution, input: std::vec::Vec<#trilogy_vm::Value>) -> std::result::Result<(), #trilogy_vm::Error> {
                let runtime = #trilogy::Runtime::new(runtime);
                let module_function = runtime.function_closure::<_, #arity>(|rt, input| {
                    let mut input = input.into_iter();
                    rt.r#return(#name(#(#inputs),*))
                });
                let arg = runtime.unlock_function(input)?;
                runtime.apply_function(module_function, arg, |rt, v| {
                    rt.r#return(v)
                })
            }

            fn arity(&self) -> usize { 2 }
        }
    })
}

pub(crate) fn impl_fn(
    function: ItemFn,
    options: Punctuated<Argument, Token![,]>,
) -> syn::Result<proc_macro2::TokenStream> {
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

    let name = &function.sig.ident;
    let vis = &function.vis;
    let arity = function.sig.inputs.len() - 1;
    if function.sig.receiver().is_some() {
        return Ok(quote! {#function});
    }

    let inputs = function.sig.inputs.iter().skip(1).map(|param| match param {
        FnArg::Typed(..) => quote! { inputs.next().unwrap() },
        _ => unreachable!(),
    });

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #name;

        impl #trilogy::NativeFunction for #name {
            fn call(&mut self, runtime: &mut #trilogy_vm::Execution, input: std::vec::Vec<#trilogy_vm::Value>) -> std::result::Result<(), #trilogy_vm::Error> {
                let runtime = #trilogy::Runtime::new(runtime);
                let module_function = runtime.function_closure::<_, #arity>(|rt, input| {
                    let mut inputs = input.into_iter();
                    #name(rt, #(#inputs),*)
                });
                let arg = runtime.unlock_function(input)?;

                #function

                runtime.apply_function(module_function, arg, |rt, v| {
                    rt.r#return(v)
                })
            }

            fn arity(&self) -> usize { 2 }
        }
    })
}
