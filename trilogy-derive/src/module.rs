use super::Options;
use quote::{format_ident, quote};
use syn::{AttrStyle, Attribute, FnArg, ImplItem, ImplItemFn, Item, ItemImpl, ItemMod, Meta, Type};

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
    let trilogy = options.trilogy();
    let trilogy_vm = options.trilogy_vm();

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
                let proc_attr = fn_item.attrs.iter().find(|item| is_proc(item) || is_func(item)).unwrap();
                let name = Options::try_from(proc_attr.clone()).unwrap().name(&fn_item.sig.ident);
                let fn_name = format_ident!("trilogy_native_method_{}", fn_item.sig.ident);
                fn_item
                    .attrs
                    .retain(|attr| !is_func(attr) && !is_proc(attr));
                if fn_item.sig.receiver().is_some() {
                    Some(quote! {
                        module = module.add_item(#name, #trilogy::NativeMethod::new(value.clone(), #fn_name));
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
                let mut module = #trilogy::NativeTypeBuilder::new(value.clone());
                #(#add_items)*
                let module = module.build();
                #trilogy_vm::Value::from(#trilogy_vm::Native::from(module))
            }
        }

        #(#native_methods)*
    })
}

fn impl_module(module: ItemMod, options: Options) -> syn::Result<proc_macro2::TokenStream> {
    let trilogy = options.trilogy();

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
            let proc_attr = fn_item
                .attrs
                .iter()
                .find(|item| is_proc(item) || is_func(item))
                .unwrap();
            let tri_name = Options::try_from(proc_attr.clone())
                .unwrap()
                .name(&fn_item.sig.ident);
            let fn_name = &fn_item.sig.ident;
            Some(quote! {
                module = module.add_item(#tri_name, #name::#fn_name);
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
    let trilogy = options.trilogy();
    let trilogy_vm = options.trilogy_vm();
    let native_name = format_ident!("trilogy_native_method_{}", fn_item.sig.ident);
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
            #vis struct #native_name;

            impl #trilogy::NativeMethodFn for #native_name {
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
    } else {
        quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #vis struct #native_name;

            impl #trilogy::NativeFunction for #native_name {
                fn call(&mut self, runtime: &mut #trilogy_vm::Execution, mut input: std::vec::Vec<#trilogy_vm::Value>) -> std::result::Result<(), #trilogy_vm::Error> {
                    let runtime = #trilogy::Runtime::new(runtime);
                    let input = runtime.unlock_procedure::<#arity>(input)?;
                    let mut input = input.into_iter();
                    #self_ty::#fn_name(runtime, #(#inputs),*)
                }

                fn arity(&self) -> usize { #arity + 1 }
            }
        }
    }
}

fn impl_func_method(
    fn_item: &ImplItemFn,
    self_ty: &Type,
    options: &Options,
) -> proc_macro2::TokenStream {
    let trilogy = options.trilogy();
    let trilogy_vm = options.trilogy_vm();
    let native_name = format_ident!("trilogy_native_method_{}", fn_item.sig.ident);
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
            #vis struct #native_name;

            impl #trilogy::NativeMethodFn for #native_name {
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
            #vis struct #native_name;

            impl #trilogy::NativeFunction for #native_name {
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
