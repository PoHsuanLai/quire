//! The `impl Token` a token family's enum gets, and the `var` (and, tuned, `input`) that need no
//! trait in scope to call.

use crate::attrs::{enum_case, variant_words};
use crate::case::slug;
use crate::token_attrs::{EnumTokens, Kind, Values, VariantTokens, enum_tokens, variant_tokens};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

/// One variant, read.
struct Member<'a> {
    ident: &'a Ident,
    var: String,
    input: Option<String>,
    values: Values,
}

/// `impl <ds-style>::tokens::token::Token for X`, or the reason `X` cannot have one.
pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let style = crate::paths::style();
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "#[derive(Token)] is for fieldless enums",
        ));
    };
    let tokens = enum_tokens(&input.attrs)?;
    let case = enum_case(&input.attrs)?;
    let mut members = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "a Token variant carries no data",
            ));
        }
        let VariantTokens {
            name,
            input,
            values,
        } = variant_tokens(&variant.attrs)?;
        let words = variant_words(&variant.attrs)?;
        let ident = &variant.ident;
        let suffix = name
            .or(words.slug)
            .unwrap_or_else(|| slug(&ident.to_string(), case));
        members.push(Member {
            ident,
            var: format!("--{}{suffix}", tokens.prefix),
            input,
            values,
        });
    }
    check(&tokens, &members, input)?;
    let name = &input.ident;
    let vars = members.iter().map(|m| {
        let (ident, var) = (m.ident, &m.var);
        quote! { #name::#ident => #style::tokens::name::VarName(#var), }
    });
    let tuned = tuned_members(name, &tokens, &members);
    let value_fn = value_fn(name, &tokens, &members);
    let prefix = &tokens.prefix;
    let kind = match tokens.kind {
        Kind::Fixed => quote! { #style::tokens::token::TokenKind::Fixed },
        Kind::Tuned => quote! { #style::tokens::token::TokenKind::Tuned },
    };
    let input_method = match tokens.kind {
        Kind::Fixed => quote! {},
        Kind::Tuned => {
            quote! { fn input(self) -> Option<#style::tokens::name::VarName> { Some(#name::input(self)) } }
        }
    };
    Ok(quote! {
        impl #name {
            /// The custom property this token is declared as.
            pub const fn var(self) -> #style::tokens::name::VarName {
                match self { #(#vars)* }
            }
            #tuned
        }

        impl #style::tokens::token::Token for #name {
            const PREFIX: &'static str = #prefix;
            const KIND: #style::tokens::token::TokenKind = #kind;

            fn var(self) -> #style::tokens::name::VarName {
                #name::var(self)
            }

            #input_method

            #value_fn
        }
    })
}

/// A tuned enum names an input for every variant; a css-path enum needs no value on a variant.
fn check(tokens: &EnumTokens, members: &[Member], input: &DeriveInput) -> syn::Result<()> {
    if members.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "a Token enum has a variant",
        ));
    }
    if tokens.kind == Kind::Tuned && members.iter().any(|m| m.input.is_none()) {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "a tuned token names its input: `#[token(input = \"--x\")]` on every variant",
        ));
    }
    if tokens.kind == Kind::Fixed && members.iter().any(|m| m.input.is_some()) {
        return Err(syn::Error::new_spanned(
            &input.ident,
            "`input` belongs to `kind = tuned`",
        ));
    }
    if tokens.css.is_none() {
        for member in members {
            resolve(&member.values, member.ident)?;
        }
    }
    Ok(())
}

/// The inherent `input`, `fallback` and `write` of a tuned enum: the property a consumer writes,
/// the value behind it, and the inline declaration that moves it.
fn tuned_members(name: &Ident, tokens: &EnumTokens, members: &[Member]) -> TokenStream {
    let style = crate::paths::style();
    if tokens.kind == Kind::Fixed {
        return quote! {};
    }
    let inputs = members.iter().filter_map(|m| {
        let (ident, input) = (m.ident, m.input.as_ref()?);
        Some(quote! { #name::#ident => #style::tokens::name::VarName(#input), })
    });
    let fallbacks = members.iter().filter_map(|m| {
        let ident = m.ident;
        let pick = resolve(&m.values, ident).ok()?;
        let value = value_expr(&pick);
        Some(quote! { #name::#ident => #value, })
    });
    quote! {
        /// The custom property a consumer writes to move this token.
        pub const fn input(self) -> #style::tokens::name::VarName {
            match self { #(#inputs)* }
        }

        /// The value the stylesheet falls back to when the consumer writes none.
        pub fn fallback(self, scope: #style::tokens::token::TokenScope) -> &'static str {
            let _ = scope;
            match self { #(#fallbacks)* }
        }

        /// The inline declaration that moves this token: `--input:value;`.
        pub fn write(self, value: &str) -> String {
            format!("{}:{value};", #name::input(self).as_str())
        }
    }
}

/// How one variant's value is picked from the scope.
enum Pick {
    Fixed(String),
    Scheme {
        light: String,
        dark: String,
    },
    Level {
        standard: String,
        others: Vec<(&'static str, String)>,
    },
    Typeface {
        system: String,
        editorial: String,
    },
}

/// Which dimension the variant's attributes name, or why they name none.
fn resolve(values: &Values, ident: &Ident) -> syn::Result<Pick> {
    let scheme = values.light.is_some() || values.dark.is_some();
    let level = values.standard.is_some() || values.reduced.is_some();
    let typeface = values.system.is_some() || values.editorial.is_some();
    let named = [values.value.is_some(), scheme, level, typeface]
        .into_iter()
        .filter(|named| *named)
        .count();
    if named != 1 {
        return Err(syn::Error::new_spanned(
            ident,
            "a variant names one of `value`, `light`+`dark`, `standard` (+ `reduced`) or `system`+`editorial`",
        ));
    }
    let both = |first: &Option<String>, second: &Option<String>| match (first, second) {
        (Some(first), Some(second)) => Ok((first.clone(), second.clone())),
        _ => Err(syn::Error::new_spanned(
            ident,
            "both values of the pair are needed",
        )),
    };
    if let Some(value) = &values.value {
        return Ok(Pick::Fixed(value.clone()));
    }
    if scheme {
        let (light, dark) = both(&values.light, &values.dark)?;
        return Ok(Pick::Scheme { light, dark });
    }
    if typeface {
        let (system, editorial) = both(&values.system, &values.editorial)?;
        return Ok(Pick::Typeface { system, editorial });
    }
    let Some(standard) = values.standard.clone() else {
        return Err(syn::Error::new_spanned(
            ident,
            "a level value needs `standard`",
        ));
    };
    let others = [("Reduced", &values.reduced)]
        .into_iter()
        .filter_map(|(level, text)| Some((level, text.clone()?)))
        .collect();
    Ok(Pick::Level { standard, others })
}

/// The one expression a variant's value is, given `scope` and the tuned input.
fn value_expr(pick: &Pick) -> TokenStream {
    let style = crate::paths::style();
    match pick {
        Pick::Fixed(value) => quote! { #value },
        Pick::Scheme { light, dark } => quote! {
            match scope.scheme {
                #style::appearance::theme::Scheme::Light => #light,
                #style::appearance::theme::Scheme::Dark => #dark,
            }
        },
        Pick::Typeface { system, editorial } => quote! {
            match scope.typeface {
                #style::appearance::typeface::Typeface::System => #system,
                #style::appearance::typeface::Typeface::Editorial => #editorial,
            }
        },
        Pick::Level { standard, others } => {
            let arms = others.iter().map(|(level, text)| {
                let level = Ident::new(level, proc_macro2::Span::call_site());
                quote! { #style::appearance::motion::MotionLevel::#level => #text, }
            });
            quote! {
                match scope.motion {
                    #(#arms)*
                    _ => #standard,
                }
            }
        }
    }
}

/// `fn css_value`: the css path's function, or a match over the variants' own values.
fn value_fn(name: &Ident, tokens: &EnumTokens, members: &[Member]) -> TokenStream {
    let style = crate::paths::style();
    if let Some(path) = &tokens.css {
        return quote! {
            fn css_value(self, scope: #style::tokens::token::TokenScope) -> #style::tokens::token::CssValue {
                #path(self, scope)
            }
        };
    }
    let arms = members.iter().filter_map(|m| {
        let ident = m.ident;
        let pick = resolve(&m.values, ident).ok()?;
        let value = value_expr(&pick);
        Some(match &m.input {
            None => quote! { #name::#ident => #style::tokens::token::CssValue::fixed(#value), },
            Some(_) => quote! {
                #name::#ident => #style::tokens::token::CssValue::tuned(#name::input(self), #name::fallback(self, scope)),
            },
        })
    });
    quote! {
        fn css_value(self, scope: #style::tokens::token::TokenScope) -> #style::tokens::token::CssValue {
            let _ = scope;
            match self { #(#arms)* }
        }
    }
}
