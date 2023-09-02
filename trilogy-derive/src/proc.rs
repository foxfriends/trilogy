use quote::quote;
use syn::{FnArg, Item};

pub(crate) fn impl_attr(item: Item) -> syn::Result<proc_macro2::TokenStream> {
    let Item::Fn(function) = item else {
        return Err(syn::Error::new_spanned(
            item,
            "this attribute may only be used on fn items",
        ));
    };

    let name = &function.sig.ident;
    let vis = &function.vis;
    let arity = function.sig.inputs.len();

    let inputs = function.sig.inputs.iter().map(|param| match param {
        FnArg::Receiver(..) => {
            quote! {
                compile_error!("a fn item used with this attribute may not have a receiver");
            }
        }
        FnArg::Typed(..) => {
            quote! { input.next().unwrap() }
        }
    });

    Ok(quote! {
        #[allow(non_camel_case_types)]
        #vis struct #name;

        impl trilogy_vm::NativeFunction for #name {
            fn call(&self, input: Vec<Value>) -> Value {
                let mut input = input.into_iter();
                #function
                #name(#(#inputs),*).into()
            }

            fn arity(&self) -> usize { #arity }
        }
    })
}
