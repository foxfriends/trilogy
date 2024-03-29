use crate::Argument;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{FnArg, Item, Token};

pub(crate) fn impl_attr(
    item: Item,
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
    let Item::Fn(function) = item else {
        return Err(syn::Error::new_spanned(
            item,
            "this attribute may only be used on fn items",
        ));
    };

    let name = &function.sig.ident;
    let vis = &function.vis;
    let arity = function.sig.inputs.len() - 1;

    if function.sig.receiver().is_some() {
        return Ok(quote! {#function});
    }

    let inputs = function.sig.inputs.iter().skip(1).map(|param| match param {
        FnArg::Receiver(..) => unreachable!(),
        FnArg::Typed(..) => quote! { input.next().unwrap() },
    });

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #name;

        impl #trilogy::NativeFunction for #name {
            fn call(&mut self, runtime: &mut #trilogy_vm::Execution, mut input: std::vec::Vec<#trilogy_vm::Value>) -> std::result::Result<(), #trilogy_vm::Error> {
                let runtime = #trilogy::Runtime::new(runtime);
                let input = runtime.unlock_procedure::<#arity>(input)?;
                let mut input = input.into_iter();

                #function

                #name(runtime, #(#inputs),*)
            }

            fn arity(&self) -> usize { #arity + 1 }
        }
    })
}
