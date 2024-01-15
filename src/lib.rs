mod convert;

use proc_macro::TokenStream;
use proc_macro_error::{abort, emit_error, proc_macro_error};
use quote::quote_spanned;
use syn::{parse_macro_input, spanned::Spanned, ItemStruct, LitStr};

/// Import a struct from a GLSL file.
/// Leave the `name` field out if your struct and the GLSL one have matching names.
///
/// # Examples
/// ```
/// #[glrs::import(path = "examples/structs.glsl", name = "Camera")]
/// struct GlCamera;
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn import(args: TokenStream, item: TokenStream) -> TokenStream {
    // Mandatory
    let mut path = None::<LitStr>;
    // Optional override
    let mut name = None::<LitStr>;

    let args_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            path = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("name") {
            name = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported import property"))
        }
    });

    parse_macro_input!(args with args_parser);

    let path = path
        .as_ref()
        .unwrap_or_else(|| abort!(path, "Missing GLSL file `path` attribute parameter"))
        .value();

    let item = parse_macro_input!(item as ItemStruct);
    let item_span = item.span();

    assert_rust_struct_validity(&item);

    let ItemStruct {
        attrs, vis, ident, ..
    } = item;

    // Use provided name or default to the struct ident.
    let gl_ident = name
        .as_ref()
        .map(|lit| lit.value())
        .unwrap_or_else(|| ident.to_string());

    let fields = convert::get_fields(path, gl_ident);

    quote_spanned! { item_span =>
        #(#attrs)*
        #vis struct #ident {
            #(#fields),*
        }
    }
    .into()
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
