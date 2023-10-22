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
    let attrs = &function.attrs;
    let arity = function.sig.inputs.len() - 1;
    let mut errors = vec![];

    if matches!(function.sig.inputs.first(), Some(FnArg::Receiver(..))) {
        errors.push(quote! {
            compile_error!("a fn item used with this attribute may not have a receiver");
        });
    }

    let inputs = function.sig.inputs.iter().skip(1).map(|param| match param {
        FnArg::Receiver(..) => unreachable!(),
        FnArg::Typed(..) => quote! { input.next().unwrap() },
    });

    Ok(quote! {
        #(#errors)*

        #[allow(non_camel_case_types)]
        #(#attrs)*
        #vis struct #name;

        impl #trilogy::NativeFunction for #name {
            fn name() -> &'static str { stringify!(#name) }

            fn call(&self, runtime: #trilogy_vm::Execution, input: Vec<Value>) -> Value {
                let mut runtime = #trilogy::Runtime::new(runtime);
                let mut input = input.into_iter();
                #function
                #name(&mut runtime, #(#inputs),*).into()
            }

            fn arity(&self) -> usize { #arity }
        }
    })
}
