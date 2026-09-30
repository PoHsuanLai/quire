//! `#[derive(Word)]` and `#[derive(Token)]`, re-exported next to the traits they implement.
//!
//! On a fieldless enum it emits `impl Word for X`: `ALL` in declaration order, `slug` (the
//! variant in kebab-case, or the enum's `#[word(case = snake)]`, or a variant's
//! `#[word(slug = "x")]`) and `label` (the variant's words in sentence case, or a variant's
//! `#[word(label = "x")]`). `parse` is the trait's provided method. The
//! generated code names the trait through the crate the caller depends on (`proc-macro-crate`):
//! `::ds_core` (or `::ds_style` for `Token`) where the manifest names it, else `::ds::base` (or
//! `::ds::style`), so a consumer that depends on `ds` alone can derive. Each of the two crates
//! declares `extern crate self` under its own name to derive inside itself.

mod attrs;
mod case;
mod expand;
mod paths;
mod token;
mod token_attrs;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Implements `Word` for a fieldless enum.
#[proc_macro_derive(Word, attributes(word))]
pub fn derive_word(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Implements `Token` for a fieldless enum that is also a `Word`: the custom property
/// `--<prefix><slug>` (or a variant's `name`), the value the variant's attributes name, and, for
/// `kind = tuned`, the `input` a consumer writes. `css = path` names a function that computes
/// every variant's value where a table is not a literal.
#[proc_macro_derive(Token, attributes(token))]
pub fn derive_token(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match token::expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
