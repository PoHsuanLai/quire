//! `#[derive(SettingsSchema)]`, re-exported by `ds-settings` (design/22-SETTINGS.md section 9,
//! "the schema is data").
//!
//! On a settings struct it emits `impl SettingsSchema for X { fn schema() -> Schema }`, one
//! `KeySpec` per field, built from `#[settings(...)]` and the field's own type
//! (`crate::gen_struct`). On a fieldless enum it emits `impl SchemaVariants for X`, so a struct
//! whose field is that enum can ask it for its variant words at run time (`crate::gen_enum`).
//! Every malformed `#[settings(...)]` is caught in `crate::attrs`, which is unit-tested
//! directly — there is no `trybuild` in this workspace's lockfile to drive a UI test instead.

mod attrs;
mod case;
mod gen_enum;
mod gen_struct;
mod shape;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(SettingsSchema, attributes(settings))]
pub fn derive_settings_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let expanded = match &input.data {
        syn::Data::Struct(data) => gen_struct::expand(&input, data),
        syn::Data::Enum(data) => gen_enum::expand(&input, data),
        syn::Data::Union(data) => Err(syn::Error::new(
            syn::spanned::Spanned::span(&data.union_token),
            "#[derive(SettingsSchema)] supports structs and fieldless enums, not unions",
        )),
    };
    match expanded {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
