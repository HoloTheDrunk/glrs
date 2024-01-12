use glsl::{
    parser::Parse,
    syntax::{
        self, ArraySpecifier, ArraySpecifierDimension, ArrayedIdentifier, StructFieldSpecifier,
        StructSpecifier, TypeSpecifierNonArray,
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

// FIXME: return a syn Path instead of an Ident
macro_rules! type_map {
    (@primitive $glsl:ident => $primitive:ty) => {
        TypeSpecifierNonArray::$glsl => $primitive
    };

    ($($glsl:ident => $lit:path),* $(,)?) => {
        /// Get the glam type corresponding to a given glsl type.
        fn map_glsl_type(ty: TypeSpecifierNonArray) -> proc_macro2::Ident {
            let span = Span::call_site();
            match ty {
                TypeSpecifierNonArray::Int => format_ident!(stringify!(i32), span = span),
                TypeSpecifierNonArray::UInt => format_ident!(stringify!(u32), span = span),
                TypeSpecifierNonArray::Float => format_ident!(stringify!(f32), span = span),
                TypeSpecifierNonArray::Double => format_ident!(stringify!(f64), span = span),
                $(TypeSpecifierNonArray::$glsl => format_ident!(stringify!(glam::$lit), span = span),)*
                ty => abort!(Span::call_site(), "Incompatible GLSL type: {:?}", ty),
            }
        }
    };
}

// Could probably figure out a more succint way to do this using grouping based on glam type
// module but this works well enough.
type_map! {
    Vec2 => f32::Vec2,
    Vec3 => f32::Vec3,
    Vec4 => f32::Vec4,

    Mat2 => f32::Mat2,
    Mat3 => f32::Mat3,
    Mat4 => f32::Mat4,

    DVec2 => f64::DVec2,
    DVec3 => f64::DVec3,
    DVec4 => f64::DVec4,

    DMat2 => f64::DMat2,
    DMat3 => f64::DMat3,
    DMat4 => f64::DMat4,

    BVec2 => bool::BVec2,
    BVec3 => bool::BVec3,
    BVec4 => bool::BVec4,

    IVec2 => i32::IVec2,
    IVec3 => i32::IVec3,
    IVec4 => i32::IVec4,

    UVec2 => u32::UVec2,
    UVec3 => u32::UVec3,
    UVec4 => u32::UVec4,
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
            // FIXME: Move Path creation to function
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
