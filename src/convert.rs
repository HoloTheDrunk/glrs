use glsl::{
    parser::Parse,
    syntax::{
        ArraySpecifier, ArraySpecifierDimension, ArrayedIdentifier, Expr, StructFieldSpecifier,
        StructSpecifier, TypeSpecifierNonArray, UnaryOp,
    },
    visitor::{Host, Visit, Visitor},
};
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site};
use quote::{format_ident, quote, ToTokens};
use syn_path::path;

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

macro_rules! type_map {
    ($($glsl:ident),* $(,)?) => {
        /// Get the glam type corresponding to a given glsl type.
        fn map_glsl_type(ty: TypeSpecifierNonArray) -> syn::Path {
            match ty {
                TypeSpecifierNonArray::Int => path!(i32),
                TypeSpecifierNonArray::UInt => path!(u32),
                TypeSpecifierNonArray::Float => path!(f32),
                TypeSpecifierNonArray::Double => path!(f64),
                $(TypeSpecifierNonArray::$glsl => path!(::glam::$glsl),)*
                ty => abort!(Span::call_site(), "Incompatible GLSL type: {:?}", ty),
            }
        }
    };
}

// Could probably figure out a more succint way to do this using grouping based on glam type
// module but this works well enough.
type_map! {
    Vec2, Vec3, Vec4,
    Mat2, Mat3, Mat4,
    DVec2, DVec3, DVec4,
    DMat2, DMat3, DMat4,
    BVec2, BVec3, BVec4,
    IVec2, IVec3, IVec4,
    UVec2, UVec3, UVec4,
}

// TODO: split into submodule
fn convert_glsl_struct_fields(fields: Vec<StructFieldSpecifier>) -> Vec<proc_macro2::TokenStream> {
    fields
        .into_iter()
        .flat_map(|field| {
            let ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: map_glsl_type(field.ty.ty),
            });

            let arr_dims = field
                .ty
                .array_specifier
                .map(|arr_spec| map_glsl_type_arr(&ty, arr_spec, None));

            field
                .identifiers
                .into_iter()
                .map(|ArrayedIdentifier { ident, array_spec }| {
                    let (name, ty) = (
                        // Name
                        format_ident!("{}", ident.to_string(), span = Span::call_site()),
                        // Array dimensions
                        array_spec
                            .map(|arr_spec| map_glsl_type_arr(&ty, arr_spec, arr_dims.clone()))
                            .unwrap_or_else(|| arr_dims.clone().unwrap_or_else(|| quote! { #ty })),
                    );

                    quote! {
                        #name: #ty
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn map_glsl_type_arr(
    ty: &syn::Type,
    array_specifier: ArraySpecifier,
    inner: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let mut res = inner;
    // Resolve backwards since the last array specifier indicates the inner-most array size.
    // Example:
    // ```glsl
    // float[2][1] my_marr[3];
    // ```
    // becomes
    // ```rs
    // my_marr: [[[f32; 1]; 2]; 3],
    // ```
    for dim in array_specifier.dimensions.into_iter().rev() {
        let size = resolve_glsl_arr_dim_size(dim);
        res = Some(res.map_or_else(
            || {
                quote! {
                    [#ty; #size]
                }
            },
            |inner| {
                quote! {
                    [#inner; #size]
                }
            },
        ));
    }
    res.expect("Invalid array dimensions mapping")
}

fn resolve_glsl_arr_dim_size(dim: ArraySpecifierDimension) -> usize {
    match dim {
        ArraySpecifierDimension::Unsized => {
            abort_call_site!("Unsized arrays are forbidden in structs.")
        }
        ArraySpecifierDimension::ExplicitlySized(expr) => {
            resolve_glsl_const_int_expr(*expr) as usize
        }
    }
}

fn resolve_glsl_const_int_expr(expr: Expr) -> u32 {
    match expr {
        Expr::IntConst(v) => {
            if v <= 0 {
                abort_call_site!("Invalid array dimension: {}", v)
            } else {
                v as u32
            }
        }
        Expr::UIntConst(v) => v,
        Expr::Unary(UnaryOp::Add, expr) => resolve_glsl_const_int_expr(*expr),
        _ => unreachable!("Unsupported GLSL const int expr"),
    }
}

pub fn get_fields(path: String, ident: String) -> Vec<proc_macro2::TokenStream> {
    let content = std::fs::read_to_string(path.clone())
        .map_err(|err| format!("Failed to open file: {err}"))
        .unwrap();
    let gl = glsl::syntax::TranslationUnit::parse(content)
        .map_err(|err| format!("Invalid OpenGL file: {err}"))
        .unwrap();

    let mut finder = StructFinder::new(ident.as_ref());
    gl.visit(&mut finder);

    let Some(Struct(StructSpecifier { fields, .. })) = finder.result else {
        abort!(path, "Could not find requested struct");
    };

    convert_glsl_struct_fields(fields.0)
}
