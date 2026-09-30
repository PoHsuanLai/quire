//! Where the generated code finds the crate that owns a trait: the crate itself, or `ds`'s facade.

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

/// The crate a derive names, as a path: `::ds_core` where the caller depends on `ds-core` (or is
/// it), else `::ds::base`. Each crate declares `extern crate self` under its own name.
pub(crate) fn base() -> TokenStream {
    home("ds-core", "base")
}

/// The path to `ds-style`: `::ds_style` where the caller depends on it (or is it), else
/// `::ds::style`.
pub(crate) fn style() -> TokenStream {
    home("ds-style", "style")
}

fn home(krate: &str, facade: &str) -> TokenStream {
    if let Some(path) = named(krate) {
        return path;
    }
    match named("ds") {
        Some(ds) => {
            let facade = Ident::new(facade, Span::call_site());
            quote! { #ds::#facade }
        }
        None => named_default(krate),
    }
}

fn named(krate: &str) -> Option<TokenStream> {
    let found = crate_name(krate).ok()?;
    let name = match found {
        FoundCrate::Itself => krate.replace('-', "_"),
        FoundCrate::Name(name) => name,
    };
    let ident = Ident::new(&name, Span::call_site());
    Some(quote! { ::#ident })
}

/// Neither crate is a dependency: the compile error names the one a manifest lacks.
fn named_default(krate: &str) -> TokenStream {
    let ident = Ident::new(&krate.replace('-', "_"), Span::call_site());
    quote! { ::#ident }
}
