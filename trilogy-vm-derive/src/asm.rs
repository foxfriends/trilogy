use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Data, DataEnum, DeriveInput, Error, Fields, Ident, LitStr, Token, Type,
};

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(raw_name);
    syn::custom_keyword!(derive);
    syn::custom_keyword!(repr);
    syn::custom_keyword!(doc);
}

enum Argument {
    Name {
        _name_token: kw::name,
        _eq_token: Token![=],
        value: Ident,
    },
    RawName {
        _name_token: kw::raw_name,
        _eq_token: Token![=],
        value: Ident,
    },
    Derive {
        _derive_token: kw::derive,
        _paren_token: token::Paren,
        derives: Punctuated<Ident, Token![,]>,
    },
    Repr {
        _repr_token: kw::repr,
        _paren_token: token::Paren,
        repr: Type,
    },
    Doc {
        _doc_token: kw::doc,
        _eq_token: Token![=],
        doc: LitStr,
    },
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            Ok(Self::Name {
                _name_token: input.parse::<kw::name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::raw_name) {
            Ok(Self::RawName {
                _name_token: input.parse::<kw::raw_name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::derive) {
            let content;
            Ok(Self::Derive {
                _derive_token: input.parse::<kw::derive>()?,
                _paren_token: parenthesized!(content in input),
                derives: content.parse_terminated(Ident::parse, Token![,])?,
            })
        } else if lookahead.peek(kw::repr) {
            let content;
            Ok(Self::Repr {
                _repr_token: input.parse::<kw::repr>()?,
                _paren_token: parenthesized!(content in input),
                repr: content.parse()?,
            })
        } else if lookahead.peek(kw::doc) {
            Ok(Self::Doc {
                _doc_token: input.parse::<kw::doc>()?,
                _eq_token: input.parse::<Token![=]>()?,
                doc: input.parse::<LitStr>()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

enum VariantArgument {
    Name {
        _name_token: kw::name,
        _eq_token: Token![=],
        value: LitStr,
    },
}

impl Parse for VariantArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::name) {
            Ok(Self::Name {
                _name_token: input.parse::<kw::name>()?,
                _eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn impl_derive(ast: DeriveInput) -> syn::Result<TokenStream> {
    let instruction = &ast.ident;
    let vis = &ast.vis;
    let attrs: Vec<Argument> = ast
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("asm"))
        .map(|attr| {
            attr.parse_args_with(|input: ParseStream| {
                input.parse_terminated(Argument::parse, Token![,])
            })
        })
        .transpose()?
        .into_iter()
        .flatten()
        .collect();
    let raw_name = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::RawName { value, .. } => Some(value.clone()),
            _ => None,
        })
        .unwrap_or(format_ident!("Raw{instruction}"));
    let name = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Name { value, .. } => Some(value.clone()),
            _ => None,
        })
        .unwrap_or(format_ident!("OpCode"));
    let doc = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Doc { doc, .. } => Some(quote! { #[doc = #doc] }),
            _ => None,
        })
        .into_iter();
    let repr = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Repr { repr, .. } => Some(quote! { #[repr(#repr)] }),
            _ => None,
        })
        .into_iter();
    let derive = attrs
        .iter()
        .find_map(|arg| match arg {
            Argument::Derive { derives, .. } => {
                let derives = derives.into_iter();
                Some(quote! { #[derive(#(#derives),*)] })
            }
            _ => None,
        })
        .into_iter();

    let mut declarations = vec![];
    let mut params = vec![];
    let mut conversions = vec![];
    let mut from_string = vec![];
    let mut from_chunk = vec![];
    let mut to_string = vec![];
    let mut instr_format = vec![];

    match ast.data {
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                let ident = &variant.ident;
                let asm = variant
                    .attrs
                    .iter()
                    .find(|attr| attr.path().is_ident("asm"))
                    .map(|attr| -> syn::Result<_> {
                        let VariantArgument::Name { value, .. } =
                            attr.parse_args::<VariantArgument>()?;
                        Ok(quote!(#value))
                    })
                    .transpose()?
                    .unwrap_or_else(|| {
                        let name = variant.ident.to_string().to_uppercase();
                        quote!(#name)
                    });
                to_string.push(quote! { Self::#ident => #asm });
                from_string.push(quote! { #asm => Self::#ident });

                let attrs = variant
                    .attrs
                    .iter()
                    .filter(|attr| !attr.path().is_ident("asm"))
                    .map(|attr| -> syn::Result<_> { Ok(quote! { #attr }) })
                    .collect::<Result<Vec<_>, _>>()?;
                declarations.push(quote! { #(#attrs)* #ident });

                match variant.fields {
                    Fields::Unit => {
                        conversions.push(quote! { #instruction::#ident => #name::#ident });
                        params.push(quote! { #name::#ident => 0 });
                        from_chunk.push(quote! { #name::#ident => #instruction::#ident });
                        instr_format.push(quote! { #instruction::#ident => self.op_code().fmt(f) });
                    }
                    Fields::Unnamed(fields) => {
                        conversions.push(quote! { #instruction::#ident(..)  => #name::#ident });
                        let count = fields.unnamed.len();
                        params.push(quote! { #name::#ident => #count });
                        let get_params = (0..count as u32).map(|_| {
                            quote! { FromChunk::from_chunk(chunk, u32::from_be(raw.param)) }
                        });
                        from_chunk.push(
                            quote! { #name::#ident => #instruction::#ident(#(#get_params),*) },
                        );
                        let bind = (0..count).map(|i| format_ident!("p{i}"));
                        let reference = bind.clone();
                        instr_format
                            .push(quote! { #instruction::#ident(#(#bind),*) => write!(f, "{: <6} {}", self.op_code(), #(#reference),*) });
                    }
                    field @ Fields::Named(..) => {
                        let error =
                            syn::Error::new_spanned(field, "record variants are not supported")
                                .into_compile_error();
                        conversions.push(quote! { Self::#ident { .. } => #error });
                        params.push(quote! { Self::#ident { .. } => #error });
                        from_chunk.push(quote! { Self::#ident { .. } => #error });
                        instr_format.push(quote! { Self::#ident { .. } => #error });
                    }
                };
            }
        }
        _ => return Err(Error::new_spanned(ast, "only enums are supported")),
    };

    Ok(quote! {
        #(#derive)*
        #(#repr)*
        #(#doc)*
        #vis enum #name {
            #(#declarations),*
        }

        #[repr(C)]
        #vis struct #raw_name {
            opcode: Offset,
            param: Offset,
        }

        impl #name {
            pub const fn params(&self)-> usize {
                match self {
                    #(#params),*
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let token = match self {
                    #(#to_string),*
                };
                token.fmt(f)
            }
        }

        #[derive(Clone, Debug)]
        pub struct OpCodeError(String);

        impl std::error::Error for OpCodeError {}
        impl std::fmt::Display for OpCodeError {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "invalid opcode `{}`", self.0)
            }
        }

        impl std::str::FromStr for #name {
            type Err = OpCodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    #(#from_string,)*
                    s => return Err(OpCodeError(s.to_owned()))
                })
            }
        }

        impl #instruction {
            pub(crate) fn from_chunk(chunk: &Chunk, offset: Offset) -> Self {
                let raw = chunk.instruction_bytes(offset);
                match OpCode::try_from(u32::from_be(raw.opcode)).unwrap() {
                    #(#from_chunk),*
                }
            }

            /// The number of bytes this instruction takes up in bytecode form.
            pub const fn byte_len(&self) -> usize { 8 }

            /// The opcode corresponding to this instruction.
            pub const fn op_code(&self) -> #name {
                match self {
                    #(#conversions),*
                }
            }
        }

        impl std::fmt::Display for #instruction {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#instr_format),*
                }
            }
        }
    })
}
