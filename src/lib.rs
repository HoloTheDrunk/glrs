use glsl::{
    parser::Parse,
    syntax::StructSpecifier,
    visitor::{Host, Visit, Visitor},
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, emit_call_site_error, emit_error, proc_macro_error};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Field, Fields, FieldsNamed,
    ItemStruct, LitStr,
};

struct Struct(StructSpecifier);

struct StructFinder {
    target: String,
    result: Option<Struct>,
}

impl StructFinder {
    fn new(target: &str) -> Self {
        Self {
            target: target.to_owned(),
            result: None,
        }
    }
}

impl Visitor for StructFinder {
    fn visit_struct_specifier(&mut self, input: &glsl::syntax::StructSpecifier) -> Visit {
        if let Some(name) = &input.name {
            if name.as_str() == self.target.as_str() {
                self.result = Some(Struct(input.clone()));
            }
        }

        Visit::Parent
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, _tokens: &mut proc_macro2::TokenStream) {
        let s = &self.0;
        for _field in &s.fields {
            todo!()
        }
    }
}

fn assert_rust_struct_validity(item: &ItemStruct) {
    if item.generics.params.len() > 0 {
        emit_error!(
            item.span(),
            "GLSL-imported structs must be free of generics"
        );
    }
    if item.fields.len() > 0 {
        emit_error!(
            item.span(),
            "GLSL-imported structs must not have any fields"
        );
    }
}

/// Import a struct from a GLSL file.
///
/// # Examples
/// ```
/// #[glrs::import(path = examples/structs.glsl)]
/// struct Camera;
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn import(args: TokenStream, item: TokenStream) -> TokenStream {
    // Mandatory
    let mut path = None::<LitStr>;
    // Optionsl override
    let mut name = None::<LitStr>;

    let args_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            Ok(path = Some(meta.value()?.parse()?))
        } else if meta.path.is_ident("name") {
            Ok(name = Some(meta.value()?.parse()?))
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

    // Search GLSL for the provided name or default to the struct ident.
    let gl_ident = name
        .map(|lit| lit.value())
        .unwrap_or_else(|| ident.to_string());

    let content = std::fs::read_to_string(path.clone()).unwrap();
    let gl = glsl::syntax::TranslationUnit::parse(content).unwrap();

    let mut finder = StructFinder::new(gl_ident.as_ref());
    gl.visit(&mut finder);

    let Some(Struct(StructSpecifier { fields, .. })) = finder.result else {
        abort!(path, "Could not find requested struct");
    };

    // TODO: Conversion of GLSL field to Rust field
    let fields = fields
        .into_iter()
        .map(|field| {
            let name = syn::Ident::new(
                field.identifiers.0[0].ident.to_string().as_ref(),
                Span::call_site(),
            );

            quote! {
                #name: i32,
            }
        })
        .collect::<Vec<_>>();

    quote_spanned! { item_span =>
        #(#attrs)*
        #vis struct #ident {
            #(#fields),*
        }
    }
    .into()
}
