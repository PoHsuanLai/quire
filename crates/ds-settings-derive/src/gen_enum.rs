//! Codegen for `#[derive(SettingsSchema)]` on an enum: `impl SchemaVariants`, the small trait a
//! struct's own derive calls to learn a field's variant words at run time
//! (design/22-SETTINGS.md section 9.1, "enums provide their variant names through a small
//! `SchemaVariants` trait the derive also implements for enums").

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::case::to_snake_case;

pub(crate) fn expand(input: &syn::DeriveInput, data: &syn::DataEnum) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let mut words = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "#[derive(SettingsSchema)] on an enum only supports fieldless variants \
                 (a settings key is a closed set of words, not a value carrying its own data)",
            ));
        }
        words.push(to_snake_case(&variant.ident.to_string()));
    }
    if words.len() < 2 {
        return Err(syn::Error::new(
            input.span(),
            "#[derive(SettingsSchema)] on an enum needs at least two variants to pick a widget",
        ));
    }
    Ok(quote! {
        impl ::ds_settings::schema::SchemaVariants for #ident {
            fn variants() -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(#words.to_owned()),*]
            }
        }
    })
}
