//! `#[derive(SettingsSchema)]`, re-exported by `ds-settings` (design/22-SETTINGS.md section 9,
//! "the schema is data").
//!
//! On a settings struct it emits `impl SettingsSchema for X { fn schema() -> Schema }`, one
//! `KeySpec` per field, built from `#[settings(...)]` and the field's own type
//! (`crate::gen_struct`): text by type (`String`, `PathBuf`, `Cow<str>`) or `#[settings(text)]`,
//! a number only with `range` (without one, a `MissingRange { field }` error). A field whose type
//! is a closed enum reads its variant words from the enum's own `Word` (`ds_core::word::Word`), so an
//! enum needs no derive of this crate's.
//! Every malformed `#[settings(...)]` is caught in `crate::attrs`, which is unit-tested
//! directly — there is no `trybuild` in this workspace's lockfile to drive a UI test instead.

mod attrs;
mod gen_row;
mod gen_struct;
mod shape;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// `#[derive(SettingsRow)]` on the struct that is one row of a `Vec<Row>` settings key: emits
/// `SettingsRow` (its columns) and `ListElement` (so a `Vec<Row>` field is a
/// `KeyKind::Rows`). Each field needs `#[settings(label = "...")]`, and a text field may add
/// `pattern = "..."`; a field is `String` (a text column) or a `Word` enum (a choice column).
#[proc_macro_derive(SettingsRow, attributes(settings))]
pub fn derive_settings_row(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match gen_row::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(SettingsSchema, attributes(settings))]
pub fn derive_settings_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = match &input.data {
        syn::Data::Struct(data) => gen_struct::expand(&input, data),
        syn::Data::Enum(data) => Err(syn::Error::new(
            syn::spanned::Spanned::span(&data.enum_token),
            "#[derive(SettingsSchema)] is for a settings struct; a settings enum derives `Word`",
        )),
        syn::Data::Union(data) => Err(syn::Error::new(
            syn::spanned::Spanned::span(&data.union_token),
            "#[derive(SettingsSchema)] supports structs, not unions",
        )),
    };
    match expanded {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
