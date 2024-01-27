mod convert;

use std::{collections::HashMap, fmt::Debug};

use proc_macro::{Literal, TokenStream};
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, spanned::Spanned, Expr, ExprLit, ItemStruct, Lit, LitStr,
    Meta, MetaNameValue,
};

struct MacroInput {
    structs: Vec<GlslImportedStruct>,
}

impl Parse for MacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut structs = Vec::new();

        while !input.is_empty() {
            structs.push(input.parse()?);
        }

        Ok(Self { structs })
    }
}

struct GlslImportedStruct {
    path: String,
    name: Option<String>,
    struct_: ItemStruct,
}

impl Debug for GlslImportedStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlslImportedStruct")
            .field("path", &self.path)
            .field(
                "name",
                &self
                    .name
                    .clone()
                    .unwrap_or_else(|| self.struct_.ident.to_string()),
            )
            .finish_non_exhaustive()
    }
}

impl Parse for GlslImportedStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut item_struct = input.parse::<ItemStruct>()?;

        // Incredibly scuffed, hopefully won't have to change it :^)
        let attrs = item_struct
            .attrs
            .iter()
            .flat_map(|attr| {
                let Meta::NameValue(mnv) = &attr.meta else {
                    return None;
                };

                let Expr::Lit(ExprLit {
                    lit: Lit::Str(ref lit),
                    ..
                }) = mnv.value
                else {
                    emit_error!(
                        attr,
                        format!(
                            "`{}` attribute value must be a literal string",
                            mnv.path.span().source_text().unwrap()
                        )
                    );
                    return Some(Err(()));
                };

                Some(Ok((mnv.path.span().source_text().unwrap(), lit.value())))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .unwrap();

        // Remove attributes "consumed" by previous step. No headache if we want more (simple) attrs later.
        item_struct.attrs.retain(|attr| {
            attrs
                .get(&attr.path().span().source_text().unwrap())
                .is_none()
        });

        Ok(Self {
            path: attrs
                .get("path")
                .expect("GLSL file `path` struct attribute is mandatory")
                .clone(),
            name: attrs.get("name").cloned(),
            struct_: item_struct,
        })
    }
}

/// Import a struct from a GLSL file.
/// Leave the `name` field out if your struct and the GLSL one have matching names.
///
/// # Examples
/// ```
/// #[glrs::import(path = "examples/structs.glsl", name = "Camera")]
/// struct GlCamera;
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn import(input: TokenStream) -> TokenStream {
    let structs = parse_macro_input!(input as MacroInput).structs;

    // TODO: Implement nested structs
    // let mut encountered = HashMap::new();

    let output = structs
        .into_iter()
        .map(
            |GlslImportedStruct {
                 path,
                 name,
                 struct_,
             }| {
                let span = struct_.span();
                assert_rust_struct_validity(&struct_);

                let ItemStruct {
                    attrs, vis, ident, ..
                } = struct_;

                let gl_ident = name.unwrap_or_else(|| ident.to_string());
                let fields = convert::get_fields(path, gl_ident);

                quote_spanned! { span =>
                    #(#attrs)*
                    #vis struct #ident {
                        #(#fields),*
                    }
                }
            },
        )
        .collect::<proc_macro2::TokenStream>();

    output.into()
}

fn assert_rust_struct_validity(item: &ItemStruct) {
    if !item.generics.params.is_empty() {
        emit_error!(
            item.span(),
            "GLSL-imported structs must be free of generics"
        );
    }
    if !item.fields.is_empty() {
        emit_error!(
            item.span(),
            "GLSL-imported structs must not have any fields"
        );
    }
}
