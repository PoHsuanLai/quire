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
    if words.is_empty() {
        return Err(syn::Error::new(
            input.span(),
            "#[derive(SettingsSchema)] on an enum needs at least one variant: a key with no \
             value cannot be stored",
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

#[cfg(test)]
mod tests {
    use super::expand;

    fn expanded(source: &str) -> syn::Result<proc_macro2::TokenStream> {
        let input: syn::DeriveInput =
            syn::parse_str(source).unwrap_or_else(|e| panic!("{source}: {e}"));
        let syn::Data::Enum(data) = &input.data else {
            panic!("{source}: not an enum");
        };
        expand(&input, data)
    }

    #[test]
    fn one_variant_is_enough() {
        let text = expanded("enum Material { Sheet }")
            .unwrap_or_else(|e| panic!("{e}"))
            .to_string();
        assert!(text.contains("\"sheet\""), "{text}");
    }

    #[test]
    fn no_variant_or_a_variant_with_data_is_an_error() {
        const CASES: &[(&str, &str)] = &[
            ("enum Nothing {}", "at least one variant"),
            ("enum Carries { A(u8), B }", "fieldless variants"),
        ];
        for (source, want) in CASES {
            let err = expanded(source).err().map(|e| e.to_string());
            assert!(
                err.as_deref().is_some_and(|e| e.contains(want)),
                "{source}: {err:?}"
            );
        }
    }
}
