use quote::quote;
use syn::punctuated::Punctuated;
use syn::{
    AttrStyle, Attribute, FnArg, ImplItem, ImplItemFn, Item, ItemImpl, ItemMod, Meta, Token, Type,
};

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
        _ => Err(syn::Error::new_spanned(
            item,
            "the `#[module]` attribute may only be used on module, struct, or impl items",
        )),
    }
}

fn impl_impl(mut module: ItemImpl, options: Options) -> syn::Result<proc_macro2::TokenStream> {
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

    let native_methods = module
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(fn_item) if fn_item.attrs.iter().any(is_proc) => {
                Some(impl_proc_method(fn_item, self_ty, &options))
            }
            ImplItem::Fn(fn_item) if fn_item.attrs.iter().any(is_func) => {
                Some(impl_func_method(fn_item, self_ty, &options))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let add_items = module.items
        .iter_mut()
        .filter_map(|item| match item {
            ImplItem::Fn(fn_item)
                if fn_item
                    .attrs
                    .iter()
                    .any(|item| is_proc(item) || is_func(item)) =>
            {
                fn_item
                    .attrs
                    .retain(|attr| !is_func(attr) && !is_proc(attr));
                let name = &fn_item.sig.ident;
                if fn_item.sig.receiver().is_some() {
                    Some(quote! {
                        module = module.add_item(stringify!(#name), #trilogy::NativeMethod::new(value.clone(), #name));
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        #module

        impl From<#self_ty> for #trilogy_vm::Value {
            fn from(value: #self_ty) -> #trilogy_vm::Value {
                let mut module = #trilogy::NativeModuleBuilder::new();
                #(#add_items)*
                let module = module.build();
                #trilogy_vm::Value::from(#trilogy_vm::Native::from(module))
            }
        }

        #(#native_methods)*
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

fn impl_proc_method(
    fn_item: &ImplItemFn,
    self_ty: &Type,
    options: &Options,
) -> proc_macro2::TokenStream {
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
    let fn_name = &fn_item.sig.ident;
    let vis = &fn_item.vis;
    let is_method = fn_item.sig.receiver().is_some();
    let non_value_params = if is_method { 2 } else { 1 };
    let inputs = fn_item
        .sig
        .inputs
        .iter()
        .skip(non_value_params)
        .map(|input| match input {
            FnArg::Typed(..) => quote!(inputs.next().unwrap()),
            _ => unreachable!(),
        });
    let arity = fn_item.sig.inputs.len() - non_value_params;

    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_type)]
        #vis struct #fn_name;

        impl #trilogy::NativeMethodFn for #fn_name {
            type SelfType = #self_ty;

            fn arity(&self) -> usize {
                #arity + 1
            }

            fn call(
                &mut self,
                receiver: &mut Self::SelfType,
                ex: &mut #trilogy_vm::Execution,
                input: Vec<#trilogy_vm::Value>,
            ) -> #trilogy::Result<()> {
                let runtime = #trilogy::Runtime::new(ex);
                let inputs = runtime.unlock_procedure::<#arity>(input)?;
                let mut inputs = inputs.into_iter();
                Self::SelfType::#fn_name(receiver.clone(), runtime, #(#inputs),*)
            }
        }
    }
}

fn impl_func_method(
    fn_item: &ImplItemFn,
    self_ty: &Type,
    options: &Options,
) -> proc_macro2::TokenStream {
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
    let fn_name = &fn_item.sig.ident;
    let vis = &fn_item.vis;
    let is_method = fn_item.sig.receiver().is_some();
    let non_value_params = if is_method { 2 } else { 1 };
    let inputs = fn_item
        .sig
        .inputs
        .iter()
        .skip(non_value_params)
        .map(|input| match input {
            FnArg::Typed(..) => quote!(inputs.next().unwrap()),
            _ => unreachable!(),
        });
    let arity = fn_item.sig.inputs.len() - non_value_params;

    if is_method {
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #fn_name;

            impl #trilogy::NativeMethodFn for #fn_name {
                type SelfType = #self_ty;

                fn arity(&self) -> usize {
                    2
                }

                fn call(
                    &mut self,
                    receiver: &mut Self::SelfType,
                    ex: &mut #trilogy_vm::Execution,
                    input: Vec<#trilogy_vm::Value>,
                ) -> #trilogy::Result<()> {
                    let runtime = #trilogy::Runtime::new(ex);
                    let receiver = receiver.clone();
                    let module_function = runtime.function_closure::<_, #arity>(move |rt, input| {
                        let mut inputs = input.into_iter();
                        Self::SelfType::#fn_name(receiver.clone(), rt, #(#inputs),*)
                    });
                    let arg = runtime.unlock_function(input)?;
                    runtime.apply_function(module_function, arg, |rt, v| {
                        rt.r#return(v)
                    })
                }
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #fn_name;

            impl #trilogy::NativeFunction for #fn_name {
                fn call(&mut self, runtime: &mut #trilogy_vm::Execution, input: std::vec::Vec<#trilogy_vm::Value>) -> std::result::Result<(), #trilogy_vm::Error> {
                    let runtime = #trilogy::Runtime::new(runtime);
                    let module_function = runtime.function_closure::<_, #arity>(|rt, input| {
                        let mut inputs = input.into_iter();
                        #self_ty::#fn_name(rt, #(#inputs),*)
                    });
                    let arg = runtime.unlock_function(input)?;
                    runtime.apply_function(module_function, arg, |rt, v| {
                        rt.r#return(v)
                    })
                }

                fn arity(&self) -> usize { 2 }
            }
        }
    }
}
