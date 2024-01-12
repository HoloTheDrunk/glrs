use glsl::{
    parser::Parse,
    syntax::{
        self, ArraySpecifier, ArraySpecifierDimension, ArrayedIdentifier, StructFieldSpecifier,
        StructSpecifier,
    },
    visitor::{Host, Visit, Visitor},
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, emit_call_site_error, emit_error, proc_macro_error};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Field, Fields, FieldsNamed,
    ItemStruct, LitStr, Token,
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

fn map_glsl_type(ty: glsl::syntax::TypeSpecifierNonArray) -> proc_macro2::Ident {
    // Oh god
    match ty {
        syntax::TypeSpecifierNonArray::Void => todo!(),
        syntax::TypeSpecifierNonArray::Bool => todo!(),
        syntax::TypeSpecifierNonArray::Int => todo!(),
        syntax::TypeSpecifierNonArray::UInt => todo!(),
        syntax::TypeSpecifierNonArray::Float => todo!(),
        syntax::TypeSpecifierNonArray::Double => todo!(),
        syntax::TypeSpecifierNonArray::Vec2 => todo!(),
        syntax::TypeSpecifierNonArray::Vec3 => todo!(),
        syntax::TypeSpecifierNonArray::Vec4 => todo!(),
        syntax::TypeSpecifierNonArray::DVec2 => todo!(),
        syntax::TypeSpecifierNonArray::DVec3 => todo!(),
        syntax::TypeSpecifierNonArray::DVec4 => todo!(),
        syntax::TypeSpecifierNonArray::BVec2 => todo!(),
        syntax::TypeSpecifierNonArray::BVec3 => todo!(),
        syntax::TypeSpecifierNonArray::BVec4 => todo!(),
        syntax::TypeSpecifierNonArray::IVec2 => todo!(),
        syntax::TypeSpecifierNonArray::IVec3 => todo!(),
        syntax::TypeSpecifierNonArray::IVec4 => todo!(),
        syntax::TypeSpecifierNonArray::UVec2 => todo!(),
        syntax::TypeSpecifierNonArray::UVec3 => todo!(),
        syntax::TypeSpecifierNonArray::UVec4 => todo!(),
        syntax::TypeSpecifierNonArray::Mat2 => todo!(),
        syntax::TypeSpecifierNonArray::Mat3 => todo!(),
        syntax::TypeSpecifierNonArray::Mat4 => todo!(),
        syntax::TypeSpecifierNonArray::Mat23 => todo!(),
        syntax::TypeSpecifierNonArray::Mat24 => todo!(),
        syntax::TypeSpecifierNonArray::Mat32 => todo!(),
        syntax::TypeSpecifierNonArray::Mat34 => todo!(),
        syntax::TypeSpecifierNonArray::Mat42 => todo!(),
        syntax::TypeSpecifierNonArray::Mat43 => todo!(),
        syntax::TypeSpecifierNonArray::DMat2 => todo!(),
        syntax::TypeSpecifierNonArray::DMat3 => todo!(),
        syntax::TypeSpecifierNonArray::DMat4 => todo!(),
        syntax::TypeSpecifierNonArray::DMat23 => todo!(),
        syntax::TypeSpecifierNonArray::DMat24 => todo!(),
        syntax::TypeSpecifierNonArray::DMat32 => todo!(),
        syntax::TypeSpecifierNonArray::DMat34 => todo!(),
        syntax::TypeSpecifierNonArray::DMat42 => todo!(),
        syntax::TypeSpecifierNonArray::DMat43 => todo!(),
        _ => todo!(),
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
        .as_ref()
        .map(|lit| lit.value())
        .unwrap_or_else(|| ident.to_string());

    let content = std::fs::read_to_string(path.clone()).unwrap();
    let gl = glsl::syntax::TranslationUnit::parse(content).unwrap();

    let mut finder = StructFinder::new(gl_ident.as_ref());
    gl.visit(&mut finder);

    let Some(Struct(StructSpecifier { fields, .. })) = finder.result else {
        abort!(path, "Could not find requested struct");
    };

    let fields = convert_glsl_struct_fields(fields.0);

    quote_spanned! { item_span =>
        #(#attrs)*
        #vis struct #ident {
            #(#fields),*
        }
    }
    .into()
}

// TODO: split into new module with helper functions for array dimensions, type conversions, etc...
fn convert_glsl_struct_fields(fields: Vec<StructFieldSpecifier>) -> Vec<proc_macro2::TokenStream> {
    fields
        .into_iter()
        .map(|field| {
            // Temporary
            let name = syn::Ident::new(
                field.identifiers.0[0].ident.to_string().as_ref(),
                Span::call_site(),
            );
            // let ty = syn::Type::new(field.ty.ty, Span::call_site());
            let ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: Some(Token![::](Span::call_site())),
                    segments: syn::punctuated::Punctuated::from_iter(
                        vec![
                            format_ident!("glam", span = Span::call_site()),
                            format_ident!("f32", span = Span::call_site()),
                            format_ident!("Vec3", span = Span::call_site()),
                        ]
                        .into_iter()
                        .map(|ident| syn::PathSegment {
                            ident,
                            arguments: syn::PathArguments::None,
                        }),
                    ),
                },
            });

            let names =
                field
                    .identifiers
                    .into_iter()
                    .map(|ArrayedIdentifier { ident, array_spec }| {
                        array_spec.map(|ArraySpecifier { dimensions }| {
                            dimensions.into_iter().map(|dim| match dim {
                                ArraySpecifierDimension::Unsized => todo!(),
                                ArraySpecifierDimension::ExplicitlySized(expr) => todo!(),
                            })
                        });
                        //
                        format_ident!("{}", ident.to_string(), span = Span::call_site())
                    });

            quote! {
                #name: #ty,
            }
        })
        .collect::<Vec<_>>()
}
