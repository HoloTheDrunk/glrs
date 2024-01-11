use proc_macro::TokenStream;
use quote::quote_spanned;
use syn::{parse_macro_input, spanned::Spanned, LitStr};

#[proc_macro_attribute]
pub fn import(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut path = None::<LitStr>;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            path = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported import property"))
        }
    });

    parse_macro_input!(args with parser);
    if path.is_none() {
        panic!("Missing path for glrs::import macro");
    }

    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_span = item.span();

    let vis = item.vis;
    let name = item.ident;

    quote_spanned! { item_span =>
        #vis struct #name;
    }
    .into()
}
