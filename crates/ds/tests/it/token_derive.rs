//! `#[derive(Token)]`: a family's custom properties and values come from its variants'
//! attributes, for every dimension a value can follow.

use ds::prelude::*;
use ds_style::tokens::name::VarName;
use ds_style::tokens::set::TokenSet;
use ds_style::tokens::token::{CssValue, Token, TokenKind, TokenScope};

/// One variant per way a value is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "fx-", kind = fixed)]
enum Fixture {
    #[token(value = "1px")]
    Flat,
    #[token(light = "#fff", dark = "#000")]
    Themed,
    #[token(standard = "300ms", reduced = "60ms")]
    Timed,
    #[token(system = "Inter", editorial = "Karla")]
    Voiced,
    #[token(name = "renamed", value = "2px")]
    TwoWords,
}

/// A tuned family: a consumer writes the input, the stylesheet falls back to the value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = tuned)]
enum Tuned {
    #[token(name = "fx-size", input = "--fx-size-px", value = "13px")]
    Size,
    #[token(name = "fx-ink", input = "--fx-ink-in", light = "#111", dark = "#eee")]
    Ink,
}

fn value(token: impl Token, scope: TokenScope) -> String {
    token.css_value(scope).to_string()
}

#[test]
fn a_variant_is_the_custom_property_prefix_plus_its_slug_or_name() {
    const CASES: &[(Fixture, &str)] = &[
        (Fixture::Flat, "--fx-flat"),
        (Fixture::Themed, "--fx-themed"),
        (Fixture::Timed, "--fx-timed"),
        (Fixture::Voiced, "--fx-voiced"),
        (Fixture::TwoWords, "--fx-renamed"),
    ];
    for (token, want) in CASES {
        assert_eq!(token.var(), VarName(want), "{token:?}");
        assert_eq!(
            Token::var(*token),
            VarName(want),
            "{token:?} through the trait"
        );
    }
    assert_eq!(Fixture::PREFIX, "fx-");
    assert_eq!(Fixture::KIND, TokenKind::Fixed);
}

#[test]
fn a_value_follows_the_dimension_its_attributes_name() {
    let base = TokenScope::BASE;
    let cases: &[(&str, Fixture, TokenScope, &str)] = &[
        ("flat", Fixture::Flat, base, "1px"),
        (
            "flat in dark",
            Fixture::Flat,
            base.in_scheme(Scheme::Dark),
            "1px",
        ),
        ("themed light", Fixture::Themed, base, "#fff"),
        (
            "themed dark",
            Fixture::Themed,
            base.in_scheme(Scheme::Dark),
            "#000",
        ),
        ("timed standard", Fixture::Timed, base, "300ms"),
        (
            "timed reduced",
            Fixture::Timed,
            base.at(MotionLevel::Reduced),
            "60ms",
        ),
        ("voiced system", Fixture::Voiced, base, "Inter"),
        (
            "voiced editorial",
            Fixture::Voiced,
            base.in_typeface(Typeface::Editorial),
            "Karla",
        ),
    ];
    for (name, token, scope, want) in cases {
        assert_eq!(value(*token, *scope), *want, "{name}");
    }
}

#[test]
fn a_tuned_token_reads_its_input_with_the_fallback_behind_it() {
    assert_eq!(Tuned::KIND, TokenKind::Tuned);
    assert_eq!(Tuned::Size.input(), VarName("--fx-size-px"));
    assert_eq!(Token::input(Tuned::Size), Some(VarName("--fx-size-px")));
    assert_eq!(
        value(Tuned::Size, TokenScope::BASE),
        "var(--fx-size-px,13px)"
    );
    assert_eq!(
        value(Tuned::Ink, TokenScope::BASE.in_scheme(Scheme::Dark)),
        "var(--fx-ink-in,#eee)"
    );
    assert_eq!(Tuned::Size.fallback(TokenScope::BASE), "13px");
    assert_eq!(Tuned::Size.write("14px"), "--fx-size-px:14px;");
    assert_eq!(Token::input(Fixture::Flat), None);
}

#[test]
fn a_set_lists_every_variable_and_every_input() {
    let fixed: Vec<VarName> = TokenSet::of::<Fixture>().vars();
    assert_eq!(fixed.len(), Fixture::ALL.len());
    let tuned = TokenSet::of::<Tuned>().vars();
    assert_eq!(
        tuned,
        [
            VarName("--fx-size"),
            VarName("--fx-size-px"),
            VarName("--fx-ink"),
            VarName("--fx-ink-in"),
        ]
    );
}

#[test]
fn a_set_declares_each_member_in_the_scope() {
    let light = TokenSet::of::<Fixture>().declarations(TokenScope::BASE);
    let names: Vec<&str> = light.iter().map(|(var, _)| var.as_str()).collect();
    assert_eq!(
        names,
        [
            "--fx-flat",
            "--fx-themed",
            "--fx-timed",
            "--fx-voiced",
            "--fx-renamed"
        ]
    );
    let dark = TokenSet::of::<Fixture>().declarations(TokenScope::BASE.in_scheme(Scheme::Dark));
    assert_ne!(light, dark);
    assert_eq!(CssValue::fixed("1px").as_str(), "1px");
}
