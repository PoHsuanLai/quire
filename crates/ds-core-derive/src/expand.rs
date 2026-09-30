//! The `impl Word` a fieldless enum gets.

use crate::attrs::{enum_case, variant_words};
use crate::case::{label, slug};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

/// `impl <ds-core>::word::Word for X`, or the reason `X` cannot have one.
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[derive(Word)] is for fieldless enums",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "a Word enum has no generics",
        ));
    }
    if data.variants.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "a Word enum has a variant",
        ));
    }
    let case = enum_case(&input.attrs)?;
    let name = &input.ident;
    let mut idents = Vec::new();
    let mut slugs = Vec::new();
    let mut labels = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "a Word variant carries no data; the enum is a closed set of words",
            ));
        }
        let words = variant_words(&variant.attrs)?;
        let ident = variant.ident.to_string();
        slugs.push(words.slug.unwrap_or_else(|| slug(&ident, case)));
        labels.push(words.label.unwrap_or_else(|| label(&ident)));
        idents.push(&variant.ident);
    }
    let base = crate::paths::base();
    Ok(quote! {
        impl #base::word::Word for #name {
            const ALL: &'static [Self] = &[#(#name::#idents),*];

            fn slug(self) -> &'static str {
                match self {
                    #(#name::#idents => #slugs,)*
                }
            }

            fn label(self) -> &'static str {
                match self {
                    #(#name::#idents => #labels,)*
                }
            }
        }
    })
}
